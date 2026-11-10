"""Validate the repository's GitHub forms and conservative public CI policy."""

import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]


def workflow_errors(spec):
    """Reject privileged triggers, mutable actions and private-data access in public CI."""
    errors = []
    if set(spec.get("on", {})) != {"pull_request", "push", "merge_group", "workflow_dispatch"}:
        errors.append("CI must use the documented unprivileged event set")
    if spec.get("permissions") != {"contents": "read"}:
        errors.append("Public CI permissions must be contents-read only")
    if "secrets." in str(spec):
        errors.append("Public CI must not reference secrets")
    for name, job in spec.get("jobs", {}).items():
        if job.get("permissions", {"contents": "read"}) != {"contents": "read"}:
            errors.append(f"{name}: job permissions must remain read-only")
        if job.get("runs-on") != "ubuntu-24.04" or not 0 < job.get("timeout-minutes", 0) <= 30:
            errors.append(f"{name}: use a bounded GitHub-hosted runner")
        for step in job.get("steps", []):
            action = step.get("uses", "")
            if action:
                if not re.fullmatch(r"actions/(?:checkout|setup-python)@[a-fA-F0-9]{40}", action):
                    errors.append(f"{name}: only reviewed, SHA-pinned checkout/setup-python actions are configured")
                inputs = step.get("with", {})
                if action.startswith("actions/checkout@") and (
                    inputs.get("persist-credentials") is not False or inputs.get("submodules") is not False
                    or "repository" in inputs or "token" in inputs
                ):
                    errors.append(f"{name}: checkout must not retain credentials or fetch another repository/submodules")
                if inputs.get("cache"):
                    errors.append(f"{name}: public CI dependency caches are not configured")
            if "${{ github.event." in step.get("run", ""):
                errors.append(f"{name}: do not interpolate event input into shell code")
    return errors


def form_errors(spec):
    """Check required form structure and unique non-markdown field identifiers."""
    errors = []
    if not spec.get("name") or not spec.get("description") or not spec.get("body"):
        errors.append("Issue form requires name, description and body")
    ids = []
    for field in spec.get("body", []):
        if field.get("type") not in {"markdown", "input", "textarea", "dropdown", "checkboxes"}:
            errors.append("Unknown issue form field type")
        if field.get("type") == "markdown":
            if not field.get("attributes", {}).get("value"):
                errors.append("Markdown field requires content")
        else:
            identity = field.get("id", "")
            if not re.fullmatch(r"[a-zA-Z0-9_-]+", identity) or identity in ids:
                errors.append("Issue form field IDs must be valid and unique")
            ids.append(identity)
            if not field.get("attributes", {}).get("label"):
                errors.append("Input field requires a label")
    return errors


def main():
    """Parse every GitHub YAML file with a safe loader and validate current contracts."""
    errors = []
    for path in (ROOT / ".github").rglob("*.yml"):
        spec = yaml.safe_load(path.read_text())
        if not isinstance(spec, dict):
            errors.append(f"{path}: expected a YAML mapping")
            continue
        if path.parent.name == "workflows":
            errors.extend(f"{path}: {error}" for error in workflow_errors(spec))
        elif path.parent.name == "ISSUE_TEMPLATE" and path.name != "config.yml":
            errors.extend(f"{path}: {error}" for error in form_errors(spec))
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("GitHub YAML, issue form structure and public CI policy checks passed.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, yaml.YAMLError) as error:
        print(f"GitHub validation failed: {error}", file=sys.stderr)
        sys.exit(1)
