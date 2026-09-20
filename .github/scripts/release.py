"""Prepare a versioned, retryable release from a main-branch source commit."""
import datetime
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib
import urllib.error
import urllib.request

REPOSITORY = "https://github.com/LukasNiessen/ArchUnitRust"
METADATA_FILES = (
    "Cargo.toml", "Cargo.lock", "CHANGELOG.md", "README.md", "docs/index.md",
    "tests/fixtures/registry_consumer/Cargo.toml.template",
)


def git(*args):
    return subprocess.check_output(["git", *args], text=True).strip()


def version_tuple(version):
    if not re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", version):
        raise ValueError(f"Expected a stable x.y.z version, got {version!r}")
    return tuple(map(int, version.split(".")))


def select_version(current, published, head, published_head=None):
    """Honor explicit bumps; otherwise increment the latest published patch."""
    current_key = version_tuple(current)
    if current in published and published_head == head:
        return current, False
    stable = [version_tuple(v) for v in published if re.fullmatch(r"\d+\.\d+\.\d+", v)]
    if not stable or current_key > max(stable):
        return current, True
    major, minor, patch = max(stable)
    return f"{major}.{minor}.{patch + 1}", True


def fetch(url):
    request = urllib.request.Request(url, headers={
        "User-Agent": f"ArchUnitRust release automation ({REPOSITORY})",
    })
    with urllib.request.urlopen(request, timeout=45) as response:
        return response.read()


def registry_versions():
    try:
        data = json.loads(fetch("https://crates.io/api/v1/crates/archunit"))
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return []
        raise
    if (data["crate"].get("repository") or "").rstrip("/") != REPOSITORY:
        raise RuntimeError("The existing archunit crate belongs to a different repository.")
    # Include yanked versions: their version numbers cannot be reused either.
    return [version["num"] for version in data["versions"]]


def published_commit(version):
    version_tuple(version)
    archive = fetch(f"https://static.crates.io/crates/archunit/archunit-{version}.crate")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as crate:
        metadata = crate.extractfile(f"archunit-{version}/.cargo_vcs_info.json")
        if metadata is None:
            raise RuntimeError("Published crate has no source commit metadata.")
        return json.load(metadata)["git"]["sha1"]


def replace_once(content, pattern, replacement, path):
    result, count = re.subn(pattern, lambda _: replacement, content, count=1, flags=re.MULTILINE)
    if count != 1:
        raise RuntimeError(f"Missing release version field in {path}")
    return result


def update_metadata(root, old, new, source, today=None):
    today = today or datetime.datetime.now(datetime.timezone.utc).date().isoformat()
    manifest_path = root / "Cargo.toml"
    manifest = manifest_path.read_text(encoding="utf-8")
    manifest = replace_once(manifest, rf'^version = "{re.escape(old)}"$',
                            f'version = "{new}"', manifest_path)
    manifest_path.write_text(manifest, encoding="utf-8")

    lock_path = root / "Cargo.lock"
    lock = lock_path.read_text(encoding="utf-8")
    lock = replace_once(lock, rf'(\[\[package\]\]\nname = "archunit"\n)version = "{re.escape(old)}"',
                        f'[[package]]\nname = "archunit"\nversion = "{new}"', lock_path)
    lock_path.write_text(lock, encoding="utf-8")

    fixture_path = root / METADATA_FILES[-1]
    fixture = fixture_path.read_text(encoding="utf-8")
    fixture = replace_once(fixture, r'^archunit = "=[^"]+"$', f'archunit = "={new}"', fixture_path)
    fixture_path.write_text(fixture, encoding="utf-8")

    for name in ("README.md", "docs/index.md"):
        path = root / name
        content = path.read_text(encoding="utf-8")
        for before, after in (
            (f"archunit@{old}", f"archunit@{new}"),
            (f'archunit = "{old}"', f'archunit = "{new}"'),
            (f"archunit\N{GRAVE ACCENT} {old}", f"archunit\N{GRAVE ACCENT} {new}"),
            (f"ArchUnitRust {old} installs", f"ArchUnitRust {new} installs"),
        ):
            content = content.replace(before, after)
        path.write_text(content, encoding="utf-8")

    changelog_path = root / "CHANGELOG.md"
    changelog = changelog_path.read_text(encoding="utf-8")
    if f"## [{new}]" not in changelog:
        heading = (
            f"## [Unreleased]\n\n## [{new}] - {today}\n\n"
            f"- Automated release of changes through [{source[:7]}]({REPOSITORY}/commit/{source})."
        )
        changelog = changelog.replace("## [Unreleased]", heading, 1)
        changelog += f"\n[{new}]: {REPOSITORY}/releases/tag/v{new}\n"
    changelog = re.sub(r"(?m)^\[Unreleased\]: .+$",
                       f"[Unreleased]: {REPOSITORY}/compare/v{new}...HEAD", changelog)
    changelog_path.write_text(changelog, encoding="utf-8")


def recover_release_commit(source):
    """Resume a previous attempt's metadata commit without picking up other changes."""
    candidates = git("log", "origin/main", "--format=%H", "--fixed-strings",
                     "--grep", f"Release-Source: {source}").splitlines()
    for candidate in candidates:
        if git("rev-parse", f"{candidate}^") != source:
            continue
        changed = set(git("diff-tree", "--no-commit-id", "--name-only", "-r", candidate).splitlines())
        if changed and changed.issubset(METADATA_FILES):
            subprocess.run(["git", "checkout", "--detach", candidate], check=True)
            return


def main():
    source = os.environ["GITHUB_SHA"]
    if not re.fullmatch(r"[0-9a-f]{40}", source):
        raise RuntimeError("Expected a full source commit SHA.")
    recover_release_commit(source)
    root = Path.cwd()
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    if manifest["package"]["name"] != "archunit":
        raise RuntimeError("This workflow only publishes archunit.")
    current = manifest["package"]["version"]
    published = registry_versions()
    head = git("rev-parse", "HEAD")
    existing_head = published_commit(current) if current in published else None
    version, publish = select_version(current, published, head, existing_head)
    if publish:
        update_metadata(root, current, version, source)
        subprocess.run(["git", "add", "--", *METADATA_FILES], check=True)
        if subprocess.run(["git", "diff", "--cached", "--quiet"]).returncode == 1:
            subprocess.run([
                "git", "commit", "-m",
                f"chore(release): archunit v{version}\n\nRelease-Source: {source}",
            ], check=True)
        if git("status", "--porcelain"):
            raise RuntimeError("Release checkout must be clean before packaging.")
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
        output.write(f"version={version}\npublish={str(publish).lower()}\nsha={git('rev-parse', 'HEAD')}\n")
    print(f"archunit {version}: {'publish after checks' if publish else 'already published; verify only'}")


if __name__ == "__main__":
    main()
