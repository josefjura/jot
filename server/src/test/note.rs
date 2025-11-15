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

    assert_eq!(5, json.len());
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
                "lines": 2
        }))
        .await;

    response.assert_status_ok();

    let json = response.json::<Vec<Note>>();

    assert_eq!(5, json.len());
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

    assert_eq!(2, json.len());
    assert_eq!(1, json[0].id);
    assert_eq!(3, json[1].id);

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
            "term": "note"
        }))
        .await;

    let json = response.json::<Vec<Note>>();

    assert_eq!(1, json.len());
    assert_eq!(5, json[0].id);
}
