import uuid

from user_utils import create_random_user


def _unique(prefix: str) -> str:
    return f"{prefix}-{uuid.uuid4().hex[:6]}"


def _create_board(http, headers, *, name=None, description=None):
    payload = {"name": name or _unique("Board")}
    if description is not None:
        payload["description"] = description
    resp = http.post(f"{http.base_url}/api/board/create_board", headers=headers, json=payload)
    assert resp.status_code == 200, resp.text
    return resp.json()


def _create_group(http, headers, board_id, *, name=None, position="001"):
    payload = {
        "board_id": board_id,
        "name": name or _unique("Group"),
        "position": position,
    }
    resp = http.post(f"{http.base_url}/api/board/groups/create_group", headers=headers, json=payload)
    assert resp.status_code == 200, resp.text
    return resp.json()


def _create_card(http, headers, group_id, *, title=None, content=None, position="001"):
    payload = {
        "group_id": group_id,
        "title": title or _unique("Card"),
        "content": content,
        "position": position,
    }
    resp = http.post(f"{http.base_url}/api/board/cards/create_card", headers=headers, json=payload)
    assert resp.status_code == 200, resp.text
    return resp.json()


def _create_metadata(http, headers, card_id, *, key=None, value=None):
    payload = {
        "card_id": card_id,
        "key": key or _unique("meta"),
        "value": value,
    }
    resp = http.post(f"{http.base_url}/api/board/card_metadata/create", headers=headers, json=payload)
    assert resp.status_code == 200, resp.text
    return resp.json()


def _get_user_id(http, headers):
    resp = http.get(f"{http.base_url}/api/auth/me", headers=headers)
    assert resp.status_code == 200, resp.text
    data = resp.json()
    return data["id"]


def _list_board_users(http, headers, board_id):
    resp = http.get(f"{http.base_url}/api/board/users/list/{board_id}", headers=headers)
    assert resp.status_code == 200, resp.text
    return resp.json()


def test_board_groups_cards_metadata_and_users_flow(http, testuser_token_headers):
    owner_headers = testuser_token_headers

    board_id = _create_board(http, owner_headers, description="Integration board")

    group_id = _create_group(http, owner_headers, board_id, name="Initial Group", position="001")

    resp = http.post(
        f"{http.base_url}/api/board/groups/update_group",
        headers=owner_headers,
        json={
            "group_id": group_id,
            "name": "Updated Group",
            "position": "010",
        },
    )
    assert resp.status_code == 200, resp.text

    resp = http.get(f"{http.base_url}/api/board/groups/get_group/{group_id}", headers=owner_headers)
    assert resp.status_code == 200, resp.text
    group_details = resp.json()
    assert group_details["name"] == "Updated Group"
    assert group_details["position"] == "010"
    assert group_details["board_id"] == board_id

    resp = http.get(f"{http.base_url}/api/board/groups/by_board/{board_id}", headers=owner_headers)
    assert resp.status_code == 200, resp.text
    groups = resp.json()
    assert any(g["group_id"] == group_id for g in groups)

    card_id = _create_card(
        http,
        owner_headers,
        group_id,
        title="Initial Card",
        content="Card description",
        position="001",
    )

    resp = http.post(
        f"{http.base_url}/api/board/cards/update_card",
        headers=owner_headers,
        json={
            "card_id": card_id,
            "title": "Updated Card",
            "content": "Refined card content",
            "position": "005",
        },
    )
    assert resp.status_code == 200, resp.text

    resp = http.get(f"{http.base_url}/api/board/cards/get_card/{card_id}", headers=owner_headers)
    assert resp.status_code == 200, resp.text
    card_details = resp.json()
    assert card_details["title"] == "Updated Card"
    assert card_details["position"] == "005"
    assert card_details["content"] == "Refined card content"
    assert card_details["group_id"] == group_id

    resp = http.get(f"{http.base_url}/api/board/cards/by_group/{group_id}", headers=owner_headers)
    assert resp.status_code == 200, resp.text
    cards = resp.json()
    assert len(cards) == 1 and cards[0]["card_id"] == card_id

    meta_id = _create_metadata(
        http,
        owner_headers,
        card_id,
        key="priority",
        value="medium",
    )

    resp = http.post(
        f"{http.base_url}/api/board/card_metadata/update",
        headers=owner_headers,
        json={
            "meta_id": meta_id,
            "value": "high",
        },
    )
    assert resp.status_code == 200, resp.text

    resp = http.get(
        f"{http.base_url}/api/board/card_metadata/get/{meta_id}",
        headers=owner_headers,
    )
    assert resp.status_code == 200, resp.text
    metadata_details = resp.json()
    assert metadata_details["value"] == "high"
    assert metadata_details["card_id"] == card_id

    resp = http.get(
        f"{http.base_url}/api/board/card_metadata/by_card/{card_id}",
        headers=owner_headers,
    )
    assert resp.status_code == 200, resp.text
    metadata_list = resp.json()
    assert len(metadata_list) == 1 and metadata_list[0]["meta_id"] == meta_id

    resp = http.post(
        f"{http.base_url}/api/board/card_metadata/delete",
        headers=owner_headers,
        json={"meta_id": meta_id},
    )
    assert resp.status_code == 200, resp.text

    resp = http.get(
        f"{http.base_url}/api/board/card_metadata/by_card/{card_id}",
        headers=owner_headers,
    )
    assert resp.status_code == 200, resp.text
    assert resp.json() == []

    collaborator_headers = create_random_user(http)
    collaborator_id = _get_user_id(http, collaborator_headers)

    resp = http.post(
        f"{http.base_url}/api/board/users/add_user",
        headers=owner_headers,
        json={
            "board_id": board_id,
            "user_id": collaborator_id,
            "role": "Editor",
        },
    )
    assert resp.status_code == 201, resp.text

    users = _list_board_users(http, owner_headers, board_id)
    assert any(u["user_id"] == collaborator_id and u["role"] == "Editor" for u in users)

    editor_group_id = _create_group(
        http,
        collaborator_headers,
        board_id,
        name="Editor Column",
        position="050",
    )

    resp = http.post(
        f"{http.base_url}/api/board/users/update_role",
        headers=owner_headers,
        json={
            "board_id": board_id,
            "user_id": collaborator_id,
            "role": "Viewer",
        },
    )
    assert resp.status_code == 200, resp.text

    forbidden_card_payload = {
        "group_id": editor_group_id,
        "title": "Viewer Card",
        "content": None,
        "position": "100",
    }
    resp = http.post(
        f"{http.base_url}/api/board/cards/create_card",
        headers=collaborator_headers,
        json=forbidden_card_payload,
    )
    assert resp.status_code == 403, resp.text

    resp = http.post(
        f"{http.base_url}/api/board/users/remove_user",
        headers=owner_headers,
        json={
            "board_id": board_id,
            "user_id": collaborator_id,
        },
    )
    assert resp.status_code == 200, resp.text

    users = _list_board_users(http, owner_headers, board_id)
    assert all(u["user_id"] != collaborator_id for u in users)

    resp = http.post(
        f"{http.base_url}/api/board/cards/delete_card",
        headers=owner_headers,
        json={"card_id": card_id},
    )
    assert resp.status_code == 200, resp.text

    for gid in (group_id, editor_group_id):
        resp = http.post(
            f"{http.base_url}/api/board/groups/delete_group",
            headers=owner_headers,
            json={"group_id": gid},
        )
        assert resp.status_code == 200, resp.text

    resp = http.get(f"{http.base_url}/api/board/groups/by_board/{board_id}", headers=owner_headers)
    assert resp.status_code == 200, resp.text
    assert resp.json() == []
