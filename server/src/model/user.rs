use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Debug)]
pub struct UserEntity {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
}

impl From<UserEntity> for User {
    fn from(val: UserEntity) -> Self {
        User {
            id: val.id,
            name: val.name,
            email: val.email,
        }
    }
}
