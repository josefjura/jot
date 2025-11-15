use axum::http::StatusCode;
use serde_json::json;

use crate::{
    model::note::Note,
    test::{self, setup_server},
};

#[sqlx::test(fixtures("user", "note"))]
async fn note_get_all_ok(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server.get("/note").authorization_bearer(token).await;

    response.assert_status_ok();

    let json = response.json::<Vec<Note>>();

    // Alice (user_id 1) has 2 notes, not all 5
    assert_eq!(2, json.len());
    // Verify they're all owned by user 1
    assert!(json.iter().all(|note| note.user_id == 1));
}

#[sqlx::test(fixtures("user"))]
async fn note_get_by_id_bad_request(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server.get("/note/666").authorization_bearer(token).await;

    response.assert_status_not_found();
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_get_by_id_ok(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server.get("/note/1").authorization_bearer(token).await;

    response.assert_status_ok();

    let json = response.json::<Note>();

    assert_eq!(1, json.id);
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_get_by_owner_ok(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server.get("/user/note").authorization_bearer(token).await;

    response.assert_status_ok();

    let json = response.json::<Vec<Note>>();

    assert_eq!(2, json.len());
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_create_ok(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server
        .post("/note")
        .authorization_bearer(token)
        .json(&json!({
                "content": "Some note",
                "tags": ["tag1", "tag2"]
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let note = response.json::<Note>();

    assert_eq!(6, note.id);
    assert_eq!("Some note", note.content);
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_search_all_params_ok(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server
        .post("/note/search")
        .authorization_bearer(token)
        .json(&json!({
                "term": "note",
                "tag": ["tag1", "tag2"],
                "date": "today",
                "lines": 2
        }))
        .await;

    response.assert_status_ok();
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_search_lines(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server
        .post("/note/search")
        .authorization_bearer(token)
        .json(&json!({
                "tag": []
        }))
        .await;

    response.assert_status_ok();

    let json = response.json::<Vec<Note>>();

    // Alice (user_id 1) has 2 notes
    assert_eq!(2, json.len());
    // Verify they're all owned by user 1
    assert!(json.iter().all(|note| note.user_id == 1));
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_search_tag(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server
        .post("/note/search")
        .authorization_bearer(token.clone())
        .json(&json!({
            "tag": ["tag1"]
        }))
        .await;

    response.assert_status_ok();

    let json = response.json::<Vec<Note>>();

    // Only Alice's note with tag1 (note ID 1), not note 3 which belongs to Bob
    assert_eq!(1, json.len());
    assert_eq!(1, json[0].id);
    assert_eq!(1, json[0].user_id);

    let response = server
        .post("/note/search")
        .authorization_bearer(token)
        .json(&json!({
            "tag": ["tag2" ,"tag1"]
        }))
        .await;

    response.assert_status_ok();

    let json = response.json::<Vec<Note>>();

    assert_eq!(1, json.len());
    assert_eq!(1, json[0].id);
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_search_term(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await;

    let response = server
        .post("/note/search")
        .authorization_bearer(token.clone())
        .json(&json!({
            "term": "test",
            "tag": []
        }))
        .await;

    response.assert_status_ok();

    let json = response.json::<Vec<Note>>();

    // Alice has 2 notes with "test" in the content (test 1, test 2)
    assert_eq!(2, json.len());
    assert!(json.iter().all(|note| note.user_id == 1));
    assert!(json.iter().all(|note| note.content.contains("test")));
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_get_by_id_forbidden_other_user(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await; // Login as Alice (user 1)

    // Try to access Bob's note (ID 3 belongs to user 2)
    let response = server.get("/note/3").authorization_bearer(token).await;

    response.assert_status(StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("user", "note"))]
async fn note_search_only_returns_own_notes(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let token = test::login(&server).await; // Login as Alice (user 1)

    // Search for tag3 which exists in Bob's notes but not Alice's
    let response = server
        .post("/note/search")
        .authorization_bearer(token)
        .json(&json!({
            "tag": ["tag3"]
        }))
        .await;

    response.assert_status_ok();

    let json = response.json::<Vec<Note>>();

    // Alice has no notes with tag3 (only Bob has tag3)
    assert_eq!(0, json.len());
}
