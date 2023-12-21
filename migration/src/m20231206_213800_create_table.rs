use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use entity::{apothecary, apothecary_user, user};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveModelTrait, Schema, Set},
};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

macro_rules! create_table_from_entity {
    ($manager:ident, $schema:ident, $entity_module:ident) => {
        $manager
            .create_table($schema.create_table_from_entity($entity_module::Entity))
            .await?;
    };
}

macro_rules! drop_table_from_entity {
    ($manager:ident, $entity_module:ident) => {
        $manager
            .drop_table(
                sea_query::Table::drop()
                    .table($entity_module::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
    };
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);

        create_table_from_entity!(manager, schema, user);
        create_table_from_entity!(manager, schema, apothecary);
        create_table_from_entity!(manager, schema, apothecary_user);

        let db: &SchemaManagerConnection<'_> = manager.get_connection();

        user::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("Admin".to_owned()),
            email: Set("admin@email.com".to_owned()),
            password: Set(hash_password("password").map_err(|e| DbErr::Migration(e.to_string()))?),
            user_type: Set(user::UserType::Admin),
        }
        .insert(db)
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table_from_entity!(manager, apothecary_user);
        drop_table_from_entity!(manager, apothecary);
        drop_table_from_entity!(manager, user);

        Ok(())
    }
}
