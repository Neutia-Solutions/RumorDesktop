"""Validate repository links, fixture syntax and Cargo reproducibility inputs."""

import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from xml.etree import ElementTree

ROOT = Path(__file__).resolve().parents[1]


def public_path_errors(root, paths):
    """Reject known private implementation/runtime paths without reading their content."""
    errors = []
    private_roots = {"adapters", "plugins", "private", ".private", "source-adapters", "source_adapters", ".runtime", ".local", ".agent-state", "artifacts"}
    private_documents = {"adapters.md", "agent-setup-recommendations.md", "frameworks.md", "source-research.md"}
    for path in paths:
        relative = path.relative_to(root)
        name = path.name
        if (not path.resolve().is_relative_to(root.resolve())
                or name.casefold() in private_documents
                or "handoffs" in relative.parts and name != "TEMPLATE.md"
                or relative.parts[0] in private_roots or "browser-profile" in relative.parts
                or relative.parts[:2] == ("playwright", ".auth") or name == ".gitmodules"
                or name.startswith("storage-state") or name in {"Cookies", "Login Data"}
                or name.startswith(".env") and name != ".env.example"
                or re.search(r"\.(?:db|sqlite|sqlite3)(?:-|$)", name)):
            errors.append(f"Private/runtime file must not enter the public repository: {relative}")
    return errors


def local_links(path, root=ROOT):
    """Reject private-workspace file links and missing public Markdown targets."""
    errors = []
    for target in re.findall(r"\[[^\]]*\]\(([^)]+)\)", path.read_text()):
        target = target.strip("<>").split("#", 1)[0]
        if not target or re.match(r"[a-zA-Z][\w+.-]*:", target):
            continue
        resolved = (path.parent / target).resolve()
        if not resolved.is_relative_to(root.resolve()):
            errors.append(f"{path}: link leaves the public repository: {target}")
        elif not resolved.exists():
            errors.append(f"{path}: missing link target {target}")
    return errors


def fixture_errors(root):
    """Validate synthetic fixture provenance, paths and structured file syntax."""
    errors = []
    manifest = json.loads((root / "manifest.json").read_text())
    for entry in manifest:
        path = (root / entry["path"]).resolve()
        if not path.is_relative_to(root.resolve()) or not path.is_file():
            errors.append(f"Invalid fixture path: {entry['path']}")
            continue
        if entry.get("provenance") != "synthetic":
            errors.append(f"Fixture must declare synthetic provenance: {path}")
        try:
            content = path.read_text()
            if entry["kind"] == "json":
                json.loads(content)
            elif entry["kind"] == "rss":
                if "<!DOCTYPE" in content.upper() or "<!ENTITY" in content.upper():
                    raise ValueError("DTD/entity declarations are forbidden in feed fixtures")
                if ElementTree.fromstring(content).tag != "rss":
                    raise ValueError("Expected RSS root")
            elif entry["kind"] != "html":
                raise ValueError(f"Unknown fixture kind: {entry['kind']}")
        except (ValueError, ElementTree.ParseError) as error:
            errors.append(f"{path}: {error}")
    return errors


def cargo_errors(root, manifests):
    """Reject missing reproducibility inputs and floating direct dependencies."""
    if not manifests:
        return []
    errors = []
    for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
        if not (root / name).is_file():
            errors.append(f"Cargo workspace requires {name}")
    toolchain = root / "rust-toolchain.toml"
    if toolchain.is_file():
        spec = tomllib.loads(toolchain.read_text())["toolchain"]
        if not re.fullmatch(r"\d+\.\d+\.\d+", spec.get("channel", "")):
            errors.append("Rust toolchain must name an exact release")
        if not {"clippy", "rustfmt"}.issubset(spec.get("components", [])):
            errors.append("Rust toolchain requires clippy and rustfmt")
    for path in manifests:
        spec = tomllib.loads(path.read_text())
        tables = [spec, spec.get("workspace", {})]
        tables.extend(spec.get("target", {}).values())
        tables.extend({"dependencies": values} for values in spec.get("patch", {}).values())
        if "replace" in spec:
            tables.append({"dependencies": spec["replace"]})
        for table in tables:
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                for name, dependency in table.get(section, {}).items():
                    detail = {"version": dependency} if isinstance(dependency, str) else dependency
                    if detail.get("workspace"):
                        continue
                    if detail.get("package", name) == "rune" and detail.get("git", "").removesuffix(".git") != "https://github.com/anomalyco/rune":
                        errors.append(f"{path}: Rune must use anomalyco/rune, not the registry crate or an unpinned path")
                    if "path" in detail:
                        if not (path.parent / detail["path"]).resolve().is_relative_to(root.resolve()):
                            errors.append(f"{path}: {name} references a checkout outside the public repository")
                        continue
                    if "git" in detail:
                        pinned = re.fullmatch(r"[a-fA-F0-9]{40}", detail.get("rev", ""))
                        if not pinned or "branch" in detail or "tag" in detail:
                            errors.append(f"{path}: {name} requires a full Git rev, no branch/tag")
                    elif not re.fullmatch(r"=\d+\.\d+\.\d+(?:-[\w.-]+)?", detail.get("version", "")):
                        errors.append(f"{path}: {name} requires an exact =x.y.z version")
    return errors


def main():
    """Check only Git-visible project files, excluding ignored private runtime data."""
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT, check=True, capture_output=True,
    )
    paths = [ROOT / p for p in result.stdout.decode().split("\0") if p]
    errors = public_path_errors(ROOT, paths)
    staged = subprocess.run(["git", "ls-files", "--stage"], cwd=ROOT, check=True, capture_output=True, text=True)
    if any(line.startswith("160000 ") for line in staged.stdout.splitlines()):
        errors.append("Submodules/gitlinks are forbidden in the public repository")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    for path in paths:
        if path.suffix == ".md":
            errors.extend(local_links(path))
    errors.extend(fixture_errors(ROOT / "tests/fixtures"))
    errors.extend(cargo_errors(ROOT, [p for p in paths if p.name == "Cargo.toml"]))
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("Repository links, synthetic fixture syntax and applicable dependency policies passed.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"Repository check failed: {error}", file=sys.stderr)
        sys.exit(1)
