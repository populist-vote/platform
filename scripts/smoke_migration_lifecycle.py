#!/usr/bin/env python3
"""Exercise the built server against an explicitly disposable local database."""
import hashlib
import getpass
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
from urllib.parse import unquote, urlsplit
from urllib.request import Request, urlopen


def main():
    database_url = os.environ.get("DATABASE_URL", "")
    target = urlsplit(database_url)
    # Reject connection overrides as well as non-local or non-test databases.
    if (target.hostname not in {"127.0.0.1", "::1", "localhost"}
            or not target.path.startswith("/migration_safety_")
            or target.query or target.fragment):
        raise SystemExit("Requires an explicit loopback DATABASE_URL named migration_safety_*, without URL overrides")
    binary = str(Path(sys.argv[1]).resolve())
    env = dict(os.environ, ENVIRONMENT="local", JWT_SECRET="local-migration-smoke-only", RUST_LOG="warn")
    for key in ("DYNO", "MIGRATION_DATABASE_URL", "PGOPTIONS", "PGHOST", "PGHOSTADDR", "PGSERVICE", "PGSERVICEFILE"):
        env.pop(key, None)

    def sql(statement):
        result = subprocess.run(["psql", "-X", "-v", "ON_ERROR_STOP=1", "-At"],
                                input=statement, text=True, capture_output=True,
                                env=dict(env, PGHOST=target.hostname, PGPORT=str(target.port or 5432),
                                         PGDATABASE=unquote(target.path[1:]),
                                         PGUSER=unquote(target.username or getpass.getuser()),
                                         PGPASSWORD=unquote(target.password or "")), check=True)
        return result.stdout.strip()

    # Ordinary/local shells must not be able to invoke the remote release mode.
    denied = subprocess.run([binary, "migrate"], env=env, text=True, capture_output=True, timeout=10)
    assert denied.returncode != 0 and "require a Heroku release dyno" in denied.stderr
    remote = dict(env, DATABASE_URL="postgres://unused@database.example.invalid/app")
    denied = subprocess.run([binary, "migrate", "--local"], env=remote, text=True, capture_output=True, timeout=10)
    assert denied.returncode != 0 and "--local refuses remote" in denied.stderr
    bad_args = subprocess.run([binary, "invalid-command"], env=env, text=True, capture_output=True, timeout=10)
    assert bad_args.returncode != 0 and "Usage:" in bad_args.stderr

    # Advance the database AFTER compilation. The already-built binary must start.
    version = int(sql("SELECT max(version) FROM _sqlx_migrations;")) + 1
    statement = f"CREATE TABLE migration_smoke_{version} (id INTEGER);"
    checksum = hashlib.sha384(statement.encode()).hexdigest()
    sql(f"BEGIN; {statement} INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) "
        f"VALUES ({version}, 'future migration smoke', true, decode('{checksum}', 'hex'), 0); COMMIT;")
    before = sql("SELECT version || ':' || success || ':' || encode(checksum, 'hex') FROM _sqlx_migrations ORDER BY version;")
    with tempfile.TemporaryFile(mode="w+") as logs:
        for _ in range(2):
            with socket.socket() as reservation:
                reservation.bind(("127.0.0.1", 0))
                port = reservation.getsockname()[1]
            process = subprocess.Popen([binary], env=dict(env, PORT=str(port)), stdout=logs, stderr=logs)
            try:
                deadline = time.monotonic() + 20
                while True:
                    if process.poll() is not None:
                        raise AssertionError("Server exited before becoming ready")
                    try:
                        with urlopen(f"http://127.0.0.1:{port}/api/v1/health", timeout=1) as response:
                            assert json.load(response)["data"]["status"] == "ok"
                        break
                    except OSError:
                        if time.monotonic() >= deadline:
                            raise AssertionError("Server did not become ready")
                        time.sleep(0.1)
                request = Request(f"http://127.0.0.1:{port}/", data=json.dumps({"query": "{ health politicalParties { __typename } }"}).encode(), headers={"Content-Type": "application/json"})
                with urlopen(request, timeout=5) as response:
                    result = json.load(response)
                    assert not result.get("errors"), result
                    assert result["data"]["health"] is True
                    assert result["data"]["politicalParties"]
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        logs.seek(0)
        assert "newer than this binary" in logs.read()
    # Rollback release validation also accepts the newer suffix without reverting it.
    subprocess.run([binary, "migrate", "--local"], env=env, check=True, timeout=20)
    subprocess.run([binary, "migrate"], env=dict(env, DYNO="release.123"), check=True, timeout=20)
    after = sql("SELECT version || ':' || success || ':' || encode(checksum, 'hex') FROM _sqlx_migrations ORDER BY version;")
    assert before == after, "Startup or rollback modified migration history"
    print("PASS: command guards, two API restarts, database-backed GraphQL, warning, and rollback with unchanged migration history")


if __name__ == "__main__":
    main()
