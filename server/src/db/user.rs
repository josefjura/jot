use rusqlite::{params, Connection, Result};

use crate::model::user::{User, UserEntity};

pub fn read_user_by_id(
    conn: &Connection,
    user_id: &str,
) -> Result<Option<User>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, email, password FROM users WHERE id = ?"
    )?;

    let user = stmt.query_row(params![user_id], |row| {
        Ok(UserEntity {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            password: row.get(3)?,
        })
    });

    match user {
        Ok(entity) => Ok(Some(entity.into())),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}
