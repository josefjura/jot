use sqlx::SqlitePool;

use crate::{
    errors::DbError,
    model::user::{User, UserEntity},
};

pub async fn read_user_by_id(
    db: &SqlitePool,
    user_id: &str,
) -> sqlx::Result<Option<User>, DbError> {
    let user = sqlx::query_as!(
        UserEntity,
        r#"
				SELECT id, name, password, email
				FROM users
				WHERE id = ?
			"#,
        user_id
    )
    .fetch_optional(db)
    .await?;

    Ok(user.map(|u| u.into()))
}
