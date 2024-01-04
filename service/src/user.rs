use std::fmt::Display;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use entity::user::Entity;
pub use entity::user::Model as User;
use entity::DatabaseConnection;
use sea_orm::{entity::prelude::*, Set, TryIntoModel};
use tracing::debug;
use uuid::Uuid;

pub enum UserServiceError {
    InvalidCredentials,
    UserAlreadyExists,
    Anyhow(anyhow::Error),
}

impl From<DbErr> for UserServiceError {
    fn from(err: DbErr) -> Self {
        Self::Anyhow(err.into())
    }
}

impl From<anyhow::Error> for UserServiceError {
    fn from(err: anyhow::Error) -> Self {
        Self::Anyhow(err)
    }
}

impl Display for UserServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserServiceError::InvalidCredentials => write!(f, "Invalid credentials"),
            UserServiceError::UserAlreadyExists => write!(f, "User already exists"),
            UserServiceError::Anyhow(e) => write!(f, "{}", e),
        }
    }
}

pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<User>, UserServiceError> {
        Ok(Entity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn login(&self, user_login: dto::user::UserLogin) -> Result<User, UserServiceError> {
        let user = Entity::find()
            .filter(entity::user::Column::Email.contains(user_login.email))
            .one(&self.db)
            .await?
            .ok_or(UserServiceError::InvalidCredentials)?;

        let parsed_hash = PasswordHash::new(&user.password).map_err(anyhow::Error::from)?;

        Argon2::default()
            .verify_password(user_login.password.as_bytes(), &parsed_hash)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => UserServiceError::InvalidCredentials,
                _ => UserServiceError::Anyhow(anyhow::Error::from(e)),
            })?;

        Ok(user)
    }

    pub async fn register(
        &self,
        user_registration: dto::user::UserRegistration,
    ) -> Result<User, UserServiceError> {
        debug!("register({:?})", user_registration);

        let password_hash = Argon2::default()
            .hash_password(
                user_registration.password.as_bytes(),
                &SaltString::generate(&mut OsRng),
            )
            .map_err(anyhow::Error::from)?
            .to_string();

        Ok(entity::user::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(user_registration.name),
            email: Set(user_registration.email),
            password: Set(password_hash),
            user_type: Set(entity::user::UserType::User),
        }
        .insert(&self.db)
        .await
        .map_err(|e| match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => UserServiceError::UserAlreadyExists,
            _ => UserServiceError::Anyhow(anyhow::Error::from(e)),
        })?
        .try_into_model()?)
    }
}
