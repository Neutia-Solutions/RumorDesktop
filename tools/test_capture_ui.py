"""Check capture error propagation and isolation without claiming Rune validation."""

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from capture_ui import capture, verify_artifacts


class CaptureChecks(unittest.TestCase):
    """Reject false evidence and ensure the runner propagates process failures."""

    def test_missing_binary_fails(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(ValueError):
                capture("", Path(temporary))

    def test_invalid_artifacts_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            png, tree = Path(temporary) / "out.png", Path(temporary) / "tree.json"
            png.write_bytes(b"not png")
            tree.write_text("{}")
            with self.assertRaises(ValueError):
                verify_artifacts(png, tree)
            png.write_bytes(b"\x89PNG\r\n\x1a\n")
            with self.assertRaises(ValueError):
                verify_artifacts(png, tree)

    def test_child_failure_is_not_swallowed(self):
        with tempfile.TemporaryDirectory() as temporary:
            with patch("capture_ui.subprocess.run", side_effect=subprocess.CalledProcessError(2, "fixture")):
                with self.assertRaises(subprocess.CalledProcessError):
                    capture("/bin/true", Path(temporary))

    def test_capture_uses_private_directories_and_fresh_outputs(self):
        calls = []

        def render(args, **kwargs):
            calls.append((args, kwargs))
            self.assertIn("--scene", args)
            self.assertIn("fixture-inbox", args)
            self.assertEqual(kwargs["timeout"], 60)
            for key in ("XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME", "XDG_STATE_HOME"):
                self.assertTrue(Path(kwargs["env"][key]).is_relative_to(kwargs["cwd"]))
            Path(args[args.index("--snap") + 1]).write_bytes(b"\x89PNG\r\n\x1a\n")
            Path(args[args.index("--tree") + 1]).write_text(json.dumps({"key": "synthetic-root"}))

        with tempfile.TemporaryDirectory() as temporary:
            with patch("capture_ui.subprocess.run", side_effect=render):
                capture("/bin/true", Path(temporary))
            self.assertEqual(len(calls), 2)
            self.assertFalse(Path(calls[0][1]["cwd"]).exists())


if __name__ == "__main__":
    unittest.main()
