use std::fmt::Display;

use dto::page::Pageable;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use entity::apothecary::Entity;
pub use entity::apothecary::Model as Apothecary;

use crate::page::{Page, PageError};

pub enum ApothecaryServiceError {
    NotFound,
    InvalidSortColumn(String),
    Anyhow(anyhow::Error),
}

impl Display for ApothecaryServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApothecaryServiceError::NotFound => write!(f, "Apothecary not found"),
            ApothecaryServiceError::InvalidSortColumn(e) => write!(f, "Invalid sort column: {}", e),
            ApothecaryServiceError::Anyhow(e) => write!(f, "{}", e),
        }
    }
}

impl From<DbErr> for ApothecaryServiceError {
    fn from(err: DbErr) -> Self {
        Self::Anyhow(err.into())
    }
}

impl From<PageError> for ApothecaryServiceError {
    fn from(err: PageError) -> Self {
        match err {
            PageError::InvalidColumnName(e) => Self::InvalidSortColumn(e),
            PageError::InvalidDirectionName => Self::Anyhow(anyhow::anyhow!("Invalid direction")),
            PageError::DbErr(e) => Self::Anyhow(anyhow::Error::from(e)),
        }
    }
}

pub struct ApothecaryService {
    db: DatabaseConnection,
}

impl ApothecaryService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get(
        &self,
        pageable: Option<Pageable>,
    ) -> Result<Page<Apothecary>, ApothecaryServiceError> {
        Page::<Apothecary>::paginate(&self.db, Entity::find(), pageable)
            .await
            .map_err(|e| e.into())
    }
}
