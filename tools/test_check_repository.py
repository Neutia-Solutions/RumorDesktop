"""Regression tests for validation failures, not application behavior."""

import json
import tempfile
import unittest
from pathlib import Path

from check_repository import cargo_errors, fixture_errors, local_links


class RepositoryChecks(unittest.TestCase):
    """Exercise broken inputs and dependency sources without network or credentials."""

    def test_local_links_resolve_relative_to_document(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "target.md").write_text("# Target\n")
            doc = root / "guide.md"
            doc.write_text("[yes](target.md#target) [web](https://news.invalid/) [no](missing.md)")
            errors = local_links(doc, root)
            self.assertEqual(len(errors), 1)
            self.assertIn("missing.md", errors[0])

    def test_existing_private_workspace_link_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "public"
            root.mkdir()
            (root.parent / "private.md").write_text("private planning")
            doc = root / "guide.md"
            doc.write_text("[private](../private.md)")
            self.assertTrue(any("leaves the public repository" in error for error in local_links(doc, root)))

    def test_file_urls_cannot_bypass_private_link_guard(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            doc = root / "guide.md"
            for scheme in ("file", "FILE"):
                doc.write_text(f"[private]({scheme}:///private/notes.md)")
                self.assertTrue(local_links(doc, root))

    def test_reference_links_cannot_bypass_private_link_guard(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "public"
            root.mkdir()
            (root.parent / "private.md").write_text("private planning")
            doc = root / "guide.md"
            for target in ("file:///private/notes.md", "../private.md", "<../private.md>"):
                doc.write_text(f"[private][notes]\n\n[notes]: {target}\n")
                self.assertTrue(local_links(doc, root))

    def test_fixture_escape_and_non_synthetic_provenance_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "fixtures"
            root.mkdir()
            (root.parent / "private.json").write_text("{}")
            (root / "data.json").write_text("{}")
            (root / "manifest.json").write_text(json.dumps([
                {"path": "../private.json", "kind": "json", "provenance": "synthetic"},
                {"path": "data.json", "kind": "json", "provenance": "publisher"},
            ]))
            errors = fixture_errors(root)
            self.assertEqual(len(errors), 2)
            self.assertIn("Invalid fixture path", errors[0])
            self.assertIn("synthetic provenance", errors[1])

    def test_invalid_feed_and_json_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "data.json").write_text("{")
            (root / "feed.xml").write_text('<!DOCTYPE rss [<!ENTITY x "text">]><rss/>')
            (root / "manifest.json").write_text(json.dumps([
                {"path": "data.json", "kind": "json", "provenance": "synthetic"},
                {"path": "feed.xml", "kind": "rss", "provenance": "synthetic"},
            ]))
            self.assertEqual(len(fixture_errors(root)), 2)

    def test_first_manifest_requires_toolchain_and_lockfile(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / "Cargo.toml"
            manifest.write_text('[workspace]\nresolver = "3"\n')
            errors = cargo_errors(root, [manifest])
            self.assertTrue(any("Cargo.lock" in error for error in errors))
            self.assertTrue(any("rust-toolchain.toml" in error for error in errors))

    def test_pins_cover_target_dependencies_and_registry_aliases(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "Cargo.lock").write_text("version = 4\n")
            (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.99.1"\ncomponents = ["clippy", "rustfmt"]\n')
            manifest = root / "Cargo.toml"
            manifest.write_text('[workspace.dependencies]\nrune = { git = "https://github.com/anomalyco/rune", rev = "' + "a" * 40 + '" }\nserde = "=1.0.0"\n')
            self.assertEqual(cargo_errors(root, [manifest]), [])
            manifest.write_text('[target.\'cfg(unix)\'.dependencies]\nserde = "1"\nui = { package = "rune", version = "=1.0.0" }\nother = { git = "https://example.invalid/repo", branch = "main" }\nunpinned = { package = "rune", path = "../rune" }\n')
            errors = cargo_errors(root, [manifest])
            self.assertTrue(any("exact =x.y.z" in error for error in errors))
            self.assertTrue(any("anomalyco/rune" in error for error in errors))
            self.assertTrue(any("full Git rev" in error for error in errors))
            self.assertTrue(any("unpinned path" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
