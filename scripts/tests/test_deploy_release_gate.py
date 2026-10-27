"""Test deployment orchestration with command fakes; never contacts Heroku."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "deploy_staging_loop.sh"


class ReleaseGateTests(unittest.TestCase):
    def run_gate(self, status, description="Deploy deadbeef"):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commands = root / "bin"
            commands.mkdir()
            web = root / "web"
            (web / ".git").mkdir(parents=True)
            marker = root / "web-pushed"
            fakes = {
                "git": '''import os, pathlib, sys
args = sys.argv[1:]
if "rev-parse" in args: print("deadbeef")
elif "--show-current" in args: print("main")
elif "push" in args and "origin" in args:
    pathlib.Path(os.environ["GATE_TEST_MARKER"]).touch()
    sys.exit(42)  # Stop before smoke checks; this is the boundary under test.
''',
                "heroku": '''import os, sys
if "releases" in sys.argv: print(os.environ["GATE_TEST_RELEASE"])
''',
                "sleep": "pass\n",
            }
            for name, code in fakes.items():
                path = commands / name
                path.write_text(f"#!{sys.executable}\n{code}")
                path.chmod(0o755)
            env = dict(os.environ, PATH=str(commands) + os.pathsep + os.environ["PATH"],
                       WEB_REPO=str(web), WEB_DEPLOY_MODE="git", ALLOW_DIRTY="1", SKIP_PREFLIGHT="1",
                       GATE_TEST_MARKER=str(marker),
                       GATE_TEST_RELEASE=json.dumps([{"version": 1, "status": status, "description": description}]))
            result = subprocess.run(["bash", str(SCRIPT)], env=env, text=True, capture_output=True, timeout=30)
            return result.returncode, marker.exists(), result.stderr

    def test_success_allows_companion_web_deploy(self):
        code, pushed, errors = self.run_gate("succeeded")
        self.assertEqual(code, 42, errors)
        self.assertTrue(pushed)

    def test_failed_release_blocks_web_deploy(self):
        code, pushed, errors = self.run_gate("failed")
        self.assertEqual(code, 1)
        self.assertFalse(pushed)
        self.assertIn("release phase failed", errors)

    def test_superseded_release_blocks_web_deploy(self):
        code, pushed, errors = self.run_gate("succeeded", "Deploy other123")
        self.assertEqual(code, 1)
        self.assertFalse(pushed)
        self.assertIn("does not match", errors)

    def test_pending_release_timeout_blocks_web_deploy(self):
        code, pushed, errors = self.run_gate("pending")
        self.assertEqual(code, 1)
        self.assertFalse(pushed)
        self.assertIn("still pending", errors)


if __name__ == "__main__":
    unittest.main()
