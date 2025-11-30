def create_random_user(http):
    import random
    import string

    username = 'user_' + ''.join(random.choices(string.ascii_lowercase + string.digits, k=8))
    password = 'pass_' + ''.join(random.choices(string.ascii_lowercase + string.digits, k=8))

    user = {
        "username": username,
        "password": password
    }
    r = http.post(f"{http.base_url}/api/auth/signup", json=user)
    assert r.status_code == 201, r.text

    r = http.post(f"{http.base_url}/api/auth/login", json=user)
    
    assert r.status_code == 200, r.text
    token = r.json().get("token")
    assert token is not None, f"No token in login response, got: {r.text}"
    headers = {"Authorization": f"Bearer {token}"}
    return headers