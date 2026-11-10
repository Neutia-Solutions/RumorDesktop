"""Capture a fixture-only Rune binary into a private temporary app environment."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def verify_artifacts(png, tree):
    """Reject missing/invalid output; this does not prove visual or keyboard quality."""
    if not png.read_bytes().startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError(f"Invalid PNG: {png}")
    parsed = json.loads(tree.read_text())
    if not parsed or not isinstance(parsed, (dict, list)):
        raise ValueError(f"Missing Rune tree: {tree}")


def capture(binary, output):
    """Run the future fixture scene without reusing the owner's app directories."""
    if not binary or not Path(binary).is_file() or not os.access(binary, os.X_OK):
        raise ValueError("Set UI_BIN to an executable Rune client with fixture-inbox support; see docs/RUNE-VERIFICATION.md")
    binary = str(Path(binary).resolve())
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="rumor-ui-") as temporary:
        env = os.environ.copy()
        env.update({name: str(Path(temporary) / name.lower()) for name in (
            "XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME", "XDG_STATE_HOME",
        )})
        for name in ("XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME", "XDG_STATE_HOME"):
            Path(env[name]).mkdir()
        for theme in ("light", "dark"):
            png, tree = output / f"{theme}.png", output / f"{theme}.json"
            subprocess.run(
                [binary, "--scene", "fixture-inbox", "--theme", theme,
                 "--snap", str(png), "--tree", str(tree), "--at", "0"],
                cwd=temporary, env=env, check=True, timeout=60,
            )
            verify_artifacts(png, tree)


def main():
    """Create a new evidence directory so old artifacts cannot produce a pass."""
    output = ROOT / "artifacts" / "rune"
    output.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix="capture-", dir=output))
    capture(sys.argv[1] if len(sys.argv) == 2 else "", run)
    print(f"Captured Rune artifacts at {run}; keyboard assertions and visual review remain separate.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Rune capture failed: {error}", file=sys.stderr)
        sys.exit(1)
