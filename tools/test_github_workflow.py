"""Regressions for public CI privileges, disclosure paths and form structure."""

import copy
import tempfile
import unittest
from pathlib import Path

import yaml

from check_github import ROOT, form_errors, workflow_errors
from check_repository import cargo_errors, public_path_errors


class GitHubWorkflowChecks(unittest.TestCase):
    """Exercise unsafe mutations of the real configuration and private dependency paths."""

    def setUp(self):
        self.workflow = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_text())

    def test_actual_workflow_conforms(self):
        self.assertEqual(workflow_errors(self.workflow), [])

    def test_privileged_event_and_mutable_action_are_rejected(self):
        self.workflow["on"]["pull_request_target"] = None
        self.workflow["jobs"]["repository-checks"]["steps"][0]["uses"] = "actions/checkout@main"
        errors = workflow_errors(self.workflow)
        self.assertTrue(any("unprivileged" in error for error in errors))
        self.assertTrue(any("SHA-pinned" in error for error in errors))

    def test_submodule_and_secret_access_are_rejected(self):
        self.workflow["jobs"]["repository-checks"]["steps"][0]["with"]["submodules"] = True
        self.workflow["env"] = {"PRIVATE_TOKEN": "${{ secrets.PRIVATE_TOKEN }}"}
        errors = workflow_errors(self.workflow)
        self.assertTrue(any("submodules" in error for error in errors))
        self.assertTrue(any("reference secrets" in error for error in errors))

    def test_duplicate_form_ids_fail(self):
        form = yaml.safe_load((ROOT / ".github/ISSUE_TEMPLATE/bug.yml").read_text())
        self.assertEqual(form_errors(form), [])
        form["body"].append(copy.deepcopy(form["body"][1]))
        self.assertTrue(any("unique" in error for error in form_errors(form)))

    def test_private_paths_are_rejected_without_reading_files(self):
        private = ["plugins/secret.rs", "adapters/private.rs", ".gitmodules", "capture/browser-profile/Cookies", ".env", "history.sqlite3-wal"]
        self.assertEqual(len(public_path_errors(ROOT, [ROOT / p for p in private])), len(private))
        self.assertEqual(public_path_errors(ROOT, [ROOT / "tests/fixtures/feedback.json", ROOT / ".env.example"]), [])

    def test_private_sibling_cargo_path_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / "Cargo.toml"
            manifest.write_text('[dependencies]\nprivate = { path = "../RumorExampleAdapter", optional = true }\n')
            self.assertTrue(any("outside the public repository" in error for error in cargo_errors(root, [manifest])))
            manifest.write_text('[patch.crates-io]\nprivate = { path = "../RumorExampleAdapter" }\n')
            self.assertTrue(any("outside the public repository" in error for error in cargo_errors(root, [manifest])))

    def test_internal_documents_and_external_symlinks_are_rejected(self):
        paths = [ROOT / "docs/SOURCE-RESEARCH.md", ROOT / "ADAPTERS.md", ROOT / "docs/handoffs/private.md"]
        self.assertEqual(len(public_path_errors(ROOT, paths)), len(paths))
        self.assertEqual(public_path_errors(ROOT, [ROOT / "docs/handoffs/TEMPLATE.md"]), [])
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "public"
            root.mkdir()
            private = root.parent / "private.md"
            private.write_text("private planning")
            link = root / "linked.md"
            link.symlink_to(private)
            self.assertTrue(public_path_errors(root, [link]))


if __name__ == "__main__":
    unittest.main()
