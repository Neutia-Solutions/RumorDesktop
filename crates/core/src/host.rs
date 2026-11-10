//! Bounded process invocation for trusted, explicitly installed provider executables.

use std::{ffi::OsString, io, path::PathBuf, process::Stdio, time::Duration};

use nix::{
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

use crate::protocol::{MAX_FRAME_BYTES, Request, Response};

/// A trusted executable path and literal arguments; no shell is involved.
pub struct ProviderCommand {
    pub executable: PathBuf,
    pub arguments: Vec<OsString>,
    pub timeout: Duration,
}

/// Public errors omit provider payloads and potentially sensitive diagnostics.
#[derive(Debug, Error)]
pub enum HostError {
    #[error("provider executable must be an absolute path")]
    InvalidExecutable,
    #[error("provider timeout must be between zero and five minutes")]
    InvalidTimeout,
    #[error("invalid provider request: {0}")]
    InvalidRequest(&'static str),
    #[error("provider frame exceeds the protocol limit")]
    Oversized,
    #[error("provider returned no complete response frame")]
    MissingFrame,
    #[error("provider returned invalid protocol JSON")]
    InvalidJson,
    #[error("provider returned extra output")]
    TrailingOutput,
    #[error("invalid provider response: {0}")]
    InvalidResponse(&'static str),
    #[error("provider deadline expired")]
    Timeout,
    #[error("provider transport failed")]
    Transport(#[from] io::Error),
}

impl ProviderCommand {
    /// Run one request in an isolated process group, killing owned children on exit/cancel.
    /// The executable is trusted code: this process boundary is not a sandbox.
    pub async fn call(&self, request: &Request) -> Result<Response, HostError> {
        if !self.executable.is_absolute() {
            return Err(HostError::InvalidExecutable);
        }
        if self.timeout.is_zero() || self.timeout > Duration::from_secs(300) {
            return Err(HostError::InvalidTimeout);
        }
        request.validate().map_err(HostError::InvalidRequest)?;
        let mut frame = serde_json::to_vec(request).map_err(|_| HostError::InvalidJson)?;
        frame.push(b'\n');
        if frame.len() > MAX_FRAME_BYTES {
            return Err(HostError::Oversized);
        }

        let mut child = Command::new(&self.executable)
            .args(&self.arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .kill_on_drop(true)
            .spawn()?;
        let mut group = ProcessGroup(child.id().map(|id| Pid::from_raw(id as i32)));
        let input = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("missing input pipe"))?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("missing output pipe"))?;
        let exchange = async {
            let mut input = input;
            input.write_all(&frame).await?;
            input.shutdown().await?;
            drop(input);
            let mut output = BufReader::new(output);
            let bytes = read_frame(&mut output).await?;
            let mut extra = [0u8; 1];
            if output.read(&mut extra).await? != 0 {
                return Err(HostError::TrailingOutput);
            }
            let response: Response =
                serde_json::from_slice(&bytes).map_err(|_| HostError::InvalidJson)?;
            response
                .validate(request)
                .map_err(HostError::InvalidResponse)?;
            Ok(response)
        };
        let result = tokio::time::timeout(self.timeout, exchange)
            .await
            .map_err(|_| HostError::Timeout);
        group.terminate();
        let _ = tokio::time::timeout(Duration::from_secs(1), child.wait()).await;
        result?
    }
}

struct ProcessGroup(Option<Pid>);

impl ProcessGroup {
    fn terminate(&mut self) {
        if let Some(pid) = self.0.take() {
            let _ = killpg(pid, Signal::SIGKILL);
        }
    }
}

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        self.terminate();
    }
}

async fn read_frame(reader: &mut (impl AsyncBufRead + Unpin)) -> Result<Vec<u8>, HostError> {
    let mut frame = Vec::new();
    loop {
        let buffer = reader.fill_buf().await?;
        if buffer.is_empty() {
            return Err(HostError::MissingFrame);
        }
        let end = buffer.iter().position(|byte| *byte == b'\n');
        let count = end.map_or(buffer.len(), |index| index + 1);
        if frame.len() + count > MAX_FRAME_BYTES {
            return Err(HostError::Oversized);
        }
        frame.extend_from_slice(&buffer[..count]);
        reader.consume(count);
        if end.is_some() {
            return Ok(frame);
        }
    }
}
