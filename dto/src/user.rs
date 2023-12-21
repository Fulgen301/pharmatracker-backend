use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserRegistration {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum UserType {
    User,
    Apothecary,
    Admin,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub user_type: UserType,
}
