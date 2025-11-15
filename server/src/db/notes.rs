use sqlx::{Sqlite, SqlitePool};

use crate::{
    errors::DbError,
    model::note::{parse_date_filter, CreateNoteRequest, DateFilter, Note, NoteEntity},
    router::note::NoteSearchRequest,
};

pub async fn get_all(db: SqlitePool) -> Result<Vec<Note>, DbError> {
    let items = sqlx::query_as!(NoteEntity, "SELECT * FROM notes")
        .fetch_all(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get all repositories: {:?}", e);
            DbError::Unknown(e)
        })?;

    items
        .into_iter()
        .map(|item| item.try_into().map_err(DbError::EntityMapping))
        .collect()
}

pub async fn get_all_by_user(db: SqlitePool, id: i64) -> Result<Vec<Note>, DbError> {
    let items = sqlx::query_as!(
        NoteEntity,
        "SELECT notes.* FROM notes
				WHERE user_id = ?",
        id
    )
    .fetch_all(&db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get all repositories: {:?}", e);
        DbError::Unknown(e)
    })?;

    items
        .into_iter()
        .map(|item| {
            item.try_into().map_err(|e| {
                // Assuming e is a String from our TryFrom implementation
                tracing::error!("Failed to convert note entity: {:?}", e);
                DbError::EntityMapping(e)
            })
        })
        .collect()
}

pub async fn get_by_id(db: SqlitePool, id: i64) -> Result<Option<Note>, DbError> {
    let item = sqlx::query_as!(NoteEntity, "SELECT * FROM notes WHERE id = ?", id)
        .fetch_optional(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get repository by id: {:?}", e);
            DbError::Unknown(e)
        })?;

    item.map(|entity| entity.try_into().map_err(DbError::EntityMapping))
        .transpose()
}

pub async fn create(
    db: SqlitePool,
    user_id: i64,
    note: CreateNoteRequest,
) -> Result<Note, DbError> {
    let tag_value = note.tags.join(",");

    let id = sqlx::query!(
        r#"
			INSERT INTO notes (content, tags, user_id)
			VALUES (?, ?, ?)
		"#,
        note.content,
        tag_value,
        user_id
    )
    .execute(&db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create note: {:?}", e);
        DbError::Unknown(e)
    })?
    .last_insert_rowid();

    let created = get_by_id(db, id)
        .await?
        .ok_or(DbError::UnableToCreate("Note not found".to_string()))?;

    Ok(created)
}

pub async fn search(db: SqlitePool, params: NoteSearchRequest) -> Result<Vec<Note>, DbError> {
    // Start with base query
    let mut query = String::from("SELECT * FROM notes WHERE 1=1");
    let mut args = Vec::new();

    // Build query conditionally based on params
    if let Some(term) = params.term {
        query.push_str(" AND content LIKE ?");
        let search_term = format!("%{}%", term);
        args.push(search_term);
    }

    // Add tag filter (tags are comma separated)

    for tag in params.tag {
        query.push_str(" AND tags LIKE ?");
        args.push(format!("%{}%", tag));
    }

    if let Some(date_filter) = params.date {
        match parse_date_filter(&date_filter) {
            DateFilter::Today => {
                query.push_str(" AND DATE(created_at) = DATE('now')");
            }
            DateFilter::Yesterday => {
                query.push_str(" AND DATE(created_at) = DATE('now', '-1 day')");
            }
            DateFilter::Past => {
                query.push_str(" AND DATE(created_at) < DATE('now')");
            }
            DateFilter::Future => {
                query.push_str(" AND DATE(created_at) > DATE('now')");
            }
            DateFilter::LastWeek => {
                query.push_str(" AND DATE(created_at) > DATE('now', '-7 days')");
            }
            DateFilter::LastMonth => {
                query.push_str(" AND DATE(created_at) > DATE('now', '-30 days')");
            }
            DateFilter::NextWeek => {
                query.push_str(" AND DATE(created_at) < DATE('now', '+7 days')");
            }
            DateFilter::NextMonth => {
                query.push_str(" AND DATE(created_at) < DATE('now', '+30 days')");
            }
            DateFilter::Specific(date) => {
                query.push_str(" AND DATE(created_at) = ?");
                args.push(date.to_string());
            }
            DateFilter::All => {}
        }
    }

    // Add order by
    query.push_str(" ORDER BY created_at DESC");

    if let Some(limit) = params.limit {
        query.push_str(" LIMIT ?");
        args.push(limit.to_string());
    }

    // Build and execute the query
    let mut db_query = sqlx::query_as::<Sqlite, NoteEntity>(&query);

    // Bind all parameters
    for arg in args {
        db_query = db_query.bind(arg);
    }

    // Execute query
    let items = db_query.fetch_all(&db).await.map_err(|e| {
        tracing::error!("Failed to search notes: {:?}", e);
        DbError::Unknown(e)
    })?;

    // Convert entities to domain objects
    items
        .into_iter()
        .map(|item| item.try_into().map_err(DbError::EntityMapping))
        .collect()
}
