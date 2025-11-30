import pathlib
import os
import subprocess
import requests
from requests.adapters import HTTPAdapter, Retry
import time
import pytest
import sys

BASE_DIR = pathlib.Path(__file__).resolve().parents[1] # /backend/
TEST_DB = BASE_DIR / "test.db"
SERVER_PORT = int(os.getenv("SERVER_PORT", 3000))
BASE_URL = f"http://localhost:{SERVER_PORT}"

def _wait_for_ready(url: str, timeout=20.0, interval=0.25):
    """Poll health endpoint until the server is up"""
    
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            r = requests.get(url, timeout=1.5)
            if r.status_code < 500:
                return
        except Exception:
            pass
        time.sleep(interval)
    raise RuntimeError(f"Server not ready at {url}")

@pytest.fixture(scope="session")
def test_database():
    db_files = [TEST_DB, TEST_DB.with_suffix(".db-shm"), TEST_DB.with_suffix(".db-wal")]

    # setup
    for f in db_files:
        if f.exists():
            f.unlink()
    
    yield str(TEST_DB)

    # cleanup
    for f in db_files:
        if f.exists():
            f.unlink()

@pytest.fixture(scope="session")
def server(test_database):
    env = os.environ.copy()
    env["DATABASE_URL"] = f"sqlite:{test_database}"

    # Offline mode makes sqlx (the rust database bridge used) skip validation with live DB
    # The backend works fine without it, but sqlx doesn't compile otherwise
    env["SQLX_OFFLINE"] = "true"

    proc = subprocess.Popen(
        ["cargo", "run"],
        cwd=str(BASE_DIR),
        env=env,
        stdout=sys.stdout,
        stderr=sys.stderr,
        text=True,
        bufsize=1
    )

    try:
        _wait_for_ready(f"{BASE_URL}/health", timeout=30.0)
        yield BASE_URL
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()

@pytest.fixture()
def http(server):
    """
    Per-test HTTP client with cookie persistence + retries.
    """
    session = requests.Session()
    retries = Retry(
        total=5,
        backoff_factor=0.2,
        status_forcelist=(502, 503, 504),
        allowed_methods=frozenset(["GET", "POST", "PUT", "PATCH", "DELETE"])
    )
    session.mount("http://", HTTPAdapter(max_retries=retries))
    session.mount("https://", HTTPAdapter(max_retries=retries))
    session.base_url = server  # attach for convenience
    yield session
    session.close()

@pytest.fixture()
def testuser_token_headers(http):
    user = {
        "username": "testuser@test.dev",
        "password": "testuserpass123"
    }
    r = http.post(f"{http.base_url}/api/auth/signup", json=user)
    assert r.status_code == 201, r.text

    r = http.post(f"{http.base_url}/api/auth/login", json=user)
    assert r.status_code == 200, r.text
    token = r.json().get("token")
    headers = {"Authorization": f"Bearer {token}"}


    yield headers
    
    