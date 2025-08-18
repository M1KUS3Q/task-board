import requests

BASE_URL = "http://localhost:3000"

def test_signup():
    url = f"{BASE_URL}/signup"
    payload = {
        "username": "testuser",
        "password": "testpassword"
    }
    response = requests.post(url, json=payload)
    print(response.status_code)
    print(response.text)

    assert response.status_code in [200, 201, 409] # 409 for conflict if user already exists

def test_login():
    url = f"{BASE_URL}/login"
    payload = {
        "username": "testuser",
        "password": "testpassword"
    }
    response = requests.post(url, json=payload)
    print(response.status_code)
    print(response.text)

    return response.text if response.status_code == 200 else None

def access_protected_with_token(token):
    url = f"{BASE_URL}/protected"
    headers = {
        "Authorization": f"Bearer {token}"
    }
    response = requests.get(url, headers=headers)
    print(response.status_code)
    print(response.text)


def main():
    print("Starting app testing...")

    access_protected_with_token("invalid_token")  # Should fail
    test_signup()
    token = test_login()
    if token:
        access_protected_with_token(token)

if __name__ == "__main__":
    main()