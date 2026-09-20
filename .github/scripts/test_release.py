import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("release", Path(__file__).with_name("release.py"))
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def test_first_publication_keeps_manifest_version(self):
        self.assertEqual(release.select_version("0.0.1", [], "new"), ("0.0.1", True))

    def test_new_commit_gets_next_patch(self):
        self.assertEqual(
            release.select_version("0.0.1", ["0.0.1"], "new", "old"), ("0.0.2", True)
        )

    def test_retry_recognizes_published_source(self):
        self.assertEqual(
            release.select_version("0.0.2", ["0.0.1", "0.0.2"], "same", "same"),
            ("0.0.2", False),
        )

    def test_explicit_minor_bump_is_preserved(self):
        self.assertEqual(release.select_version("0.1.0", ["0.0.8"], "new"), ("0.1.0", True))

    def test_versions_sort_numerically_and_include_reserved_numbers(self):
        self.assertEqual(
            release.select_version("0.0.1", ["0.0.9", "0.0.10"], "new"), ("0.0.11", True)
        )

    def test_prereleases_do_not_replace_stable_version_order(self):
        self.assertEqual(
            release.select_version("0.0.1", ["0.0.1", "1.0.0-beta.1"], "new"), ("0.0.2", True)
        )

    def test_invalid_manifest_versions_are_rejected(self):
        for version in ("0.1", "v0.0.1", "0.0.01", "1.0.0-beta", "../secret"):
            with self.subTest(version=version), self.assertRaises(ValueError):
                release.select_version(version, [], "new")

    def test_metadata_changes_only_the_root_package_and_current_install_examples(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = {
                "Cargo.toml": '[package]\nname = "archunit"\nversion = "0.0.1"\n',
                "Cargo.lock": 'version = 4\n\n[[package]]\nname = "archunit"\nversion = "0.0.1"\n\n'
                              '[[package]]\nname = "other"\nversion = "0.0.1"\n',
                "README.md": 'cargo add --dev archunit@0.0.1\narchunit = "0.0.1"\n',
                "docs/index.md": 'ArchUnitRust 0.0.1 installs from crates.io\n',
                "CHANGELOG.md": '# Changelog\n\n## [Unreleased]\n\n## [0.0.1] - 2026-08-24\n\n'
                                '[Unreleased]: old\n',
                release.METADATA_FILES[-1]: '[dev-dependencies]\narchunit = "=0.0.1"\n',
            }
            for name, content in files.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            release.update_metadata(root, "0.0.1", "0.0.2", "a" * 40, "2026-09-20")
            lock = (root / "Cargo.lock").read_text(encoding="utf-8")
            self.assertIn('name = "archunit"\nversion = "0.0.2"', lock)
            self.assertIn('name = "other"\nversion = "0.0.1"', lock)
            self.assertIn('archunit = "=0.0.2"', (root / release.METADATA_FILES[-1]).read_text())
            self.assertIn("archunit@0.0.2", (root / "README.md").read_text())
            self.assertIn("## [0.0.2] - 2026-09-20", (root / "CHANGELOG.md").read_text())
            self.assertIn("## [0.0.1] - 2026-08-24", (root / "CHANGELOG.md").read_text())


    def test_partial_release_retry_recovers_the_same_metadata_commit(self):
        import os
        import subprocess
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "repo"
            root.mkdir()
            output = Path(directory) / "outputs"
            original_directory = Path.cwd()
            try:
                os.chdir(root)
                for name, content in {
                    "Cargo.toml": '[package]\nname = "archunit"\nversion = "0.0.1"\n',
                    "Cargo.lock": 'version = 4\n\n[[package]]\nname = "archunit"\nversion = "0.0.1"\n',
                    "README.md": 'archunit@0.0.1\n',
                    "docs/index.md": 'ArchUnitRust 0.0.1 installs\n',
                    "CHANGELOG.md": '## [Unreleased]\n\n## [0.0.1] - 2026-08-24\n[Unreleased]: old\n',
                    release.METADATA_FILES[-1]: 'archunit = "=0.0.1"\n',
                }.items():
                    path = root / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text(content, encoding="utf-8")
                for args in (
                    ("init", "-q"), ("config", "user.name", "Release Test"),
                    ("config", "user.email", "release@example.invalid"),
                    ("add", "."), ("commit", "-qm", "source"),
                ):
                    subprocess.run(["git", *args], check=True)
                source = release.git("rev-parse", "HEAD")
                subprocess.run(["git", "update-ref", "refs/remotes/origin/main", source], check=True)
                with patch.dict(os.environ, {"GITHUB_SHA": source, "GITHUB_OUTPUT": str(output)}):
                    with patch.object(release, "registry_versions", return_value=["0.0.1"]), \
                            patch.object(release, "published_commit", return_value="0" * 40):
                        release.main()
                    metadata_commit = release.git("rev-parse", "HEAD")
                    self.assertNotEqual(source, metadata_commit)
                    self.assertIn("version=0.0.2\npublish=true", output.read_text())
                    subprocess.run(["git", "update-ref", "refs/remotes/origin/main", metadata_commit], check=True)
                    subprocess.run(["git", "checkout", "-q", "--detach", source], check=True)
                    with patch.object(release, "registry_versions", return_value=["0.0.1", "0.0.2"]), \
                            patch.object(release, "published_commit", return_value=metadata_commit):
                        release.main()
                    self.assertEqual(release.git("rev-parse", "HEAD"), metadata_commit)
                    self.assertIn("version=0.0.2\npublish=false", output.read_text())
            finally:
                os.chdir(original_directory)


if __name__ == "__main__":
    unittest.main()
