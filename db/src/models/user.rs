use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: Role,
}

#[derive(Debug, Clone)]
pub enum Role {
    SUPERUSER,
    STAFF,
    BASIC,
}
