def test_signup_login_and_protected(http):
    # 1) Signup
    r = http.post(f"{http.base_url}/api/auth/signup", json={
        "username": "new@user.dev",
        "password": "S3cure!pass"
    })
    assert r.status_code == 201, r.text

    # 2) Login (assumes cookie or token returned)
    r = http.post(f"{http.base_url}/api/auth/login", json={
        "username": "new@user.dev",
        "password": "S3cure!pass"
    })
    assert r.status_code == 200, r.text

    # If your API returns a JWT instead of cookie, store it:
    token = r.json().get("token")
    assert token is not None, f"No token in login response, got: {r.text}"
    headers = {"Authorization": f"Bearer {token}"}

    # 3) Hit a protected route
    r = http.get(f"{http.base_url}/api/me", headers=headers)
    assert r.status_code == 200, r.text
    body = r.json()
    assert body["username"] == "new@user.dev"
