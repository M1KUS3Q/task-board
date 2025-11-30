from user_utils import create_random_user
def test_board_crud(http, testuser_token_headers):
    r = http.post(f"{http.base_url}/api/board/create_board", headers=testuser_token_headers, json={
        "name": "My First Board",
    })

    assert r.status_code == 200, r.text
    test_board_id = r.json()
    assert test_board_id is not None

    r = http.get(f"{http.base_url}/api/board/get_board/{test_board_id}", headers=testuser_token_headers)
    assert r.status_code == 200, r.text
    board_details = r.json()
    assert board_details
    assert board_details["name"] == "My First Board"
    assert not board_details["description"]
    
    # Try deleting without authentication
    r = http.post(f"{http.base_url}/api/board/delete_board", json={
        "board_id": test_board_id
    })

    # here 400 is expected because axum can't match the route without auth header, normally it would be 401
    assert r.status_code == 400, r.text

    unauthorized_headers = create_random_user(http)  # Create a new user to ensure different user

    # Try deleting with another user
    r = http.post(f"{http.base_url}/api/board/delete_board", headers=unauthorized_headers, json={
        "board_id": test_board_id
    })

    # here 403 is expected because the user is authorized for this page but not allowed to delete this board
    assert r.status_code == 403, r.text

    # Try changing name with another user
    r = http.post(f"{http.base_url}/api/board/update_board", headers=unauthorized_headers, json={
        "board_id": test_board_id,
        "name": "Hacked Board Name"
    })

    # here 403 is expected because the user is authorized for this page but not allowed to update this board
    assert r.status_code == 403, r.text

    # Change name with the correct user but no fields
    r = http.post(f"{http.base_url}/api/board/update_board", headers=testuser_token_headers, json={
        "board_id": test_board_id,
    })

    assert r.status_code == 400, r.text

    # Change name with the correct user
    r = http.post(f"{http.base_url}/api/board/update_board", headers=testuser_token_headers, json={
        "board_id": test_board_id,
        "name": "Updated Board Name"
    })

    assert r.status_code == 200, r.text

    # Change description with the correct user
    r = http.post(f"{http.base_url}/api/board/update_board", headers=testuser_token_headers, json={
        "board_id": test_board_id,
        "description": "Updated Description"
    })

    assert r.status_code == 200, r.text

    # Fetch board details
    r = http.get(f"{http.base_url}/api/board/get_board/{test_board_id}", headers=testuser_token_headers)
    assert r.status_code == 200, r.text
    board_details = r.json()
    assert board_details
    assert board_details["name"] == "Updated Board Name"
    assert board_details["description"] == "Updated Description"
    

    # Now delete with authentication
    r = http.post(f"{http.base_url}/api/board/delete_board", headers=testuser_token_headers, json={
        "board_id": test_board_id
    })

    assert r.status_code == 200, r.text

