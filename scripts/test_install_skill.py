import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from install_skill import SOURCE, install, skills_root

SCRIPT = Path(__file__).with_name("install_skill.py").resolve()


class InstallSkillTests(unittest.TestCase):
    def test_agent_paths(self):
        with patch("pathlib.Path.home", return_value=Path("/fake-home")), patch.dict(os.environ, {"XDG_CONFIG_HOME": "/xdg"}):
            expected = {
                "common": "/fake-home/.agents/skills", "codex": "/fake-home/.agents/skills",
                "claude": "/fake-home/.claude/skills", "gemini": "/fake-home/.gemini/skills",
                "opencode": "/xdg/opencode/skills",
            }
            for agent, path in expected.items():
                self.assertEqual(skills_root(agent, "user", Path("/project")), Path(path))
                folder = {"common": ".agents", "codex": ".agents", "claude": ".claude", "gemini": ".gemini", "opencode": ".opencode"}[agent]
                self.assertEqual(skills_root(agent, "project", Path("/project")), Path("/project") / folder / "skills")

    def test_standalone_install_and_explicit_update(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "skills with spaces"
            destination = install(root)
            for source in SOURCE.rglob("*"):
                if source.is_file():
                    self.assertEqual((destination / source.relative_to(SOURCE)).read_bytes(), source.read_bytes())
            edited = destination / "SKILL.md"
            edited.write_text("local edits")
            with self.assertRaises(ValueError):
                install(root)
            self.assertEqual(edited.read_text(), "local edits")
            neighbor = root / "another-skill"
            neighbor.mkdir()
            (destination / "obsolete.txt").write_text("old")
            install(root, force=True)
            self.assertEqual(edited.read_bytes(), (SOURCE / "SKILL.md").read_bytes())
            self.assertFalse((destination / "obsolete.txt").exists())
            self.assertTrue(neighbor.exists())

    def test_cli_from_another_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = subprocess.run([sys.executable, str(SCRIPT), "--agent", "claude", "--scope", "project", "--project", str(root / "project")], cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((root / "project/.claude/skills/rls/references/formats.md").is_file())
            result = subprocess.run([sys.executable, str(SCRIPT), "--dest", str(root / "custom")], cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((root / "custom/rls/SKILL.md").is_file())

    def test_symlink_and_source_are_not_overwritten(self):
        with self.assertRaises(ValueError):
            install(SOURCE.parent, force=True)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "rls").symlink_to(SOURCE, target_is_directory=True)
            with self.assertRaises(ValueError):
                install(root, force=True)

    def test_unknown_agent_is_an_error(self):
        result = subprocess.run([sys.executable, str(SCRIPT), "--agent", "unknown"], capture_output=True)
        self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
