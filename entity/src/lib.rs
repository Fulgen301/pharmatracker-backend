pub mod apothecary;
pub mod apothecary_medication;
pub mod apothecary_user;
pub mod medication;
pub mod user;

pub use sea_orm::DatabaseConnection;

use sea_orm::Database;

pub async fn create_database_connection(
    db_url: &str,
) -> anyhow::Result<sea_orm::DatabaseConnection> {
    Ok(Database::connect(db_url).await?)
}
