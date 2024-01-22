use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use entity::{apothecary, apothecary_medication, apothecary_user, medication, user};
use rust_decimal::Decimal;
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
        create_table_from_entity!(manager, schema, medication);
        create_table_from_entity!(manager, schema, apothecary_medication);

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

        let apothecary_id = apothecary::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("St. Rudolf".to_owned()),
            longitude: Set(16.3181194),
            latitude: Set(48.1942566),
            street: Set("Goldschlagstraße".to_owned()),
            number: Set("105".to_owned()),
            post_code: Set(1150),
            city: Set("Wien".to_owned()),
            country: Set("AT".to_owned()),
        }
        .insert(db)
        .await?
        .id;

        let medication_id = medication::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("Ibuprofen".to_owned()),
        }
        .insert(db)
        .await?
        .id;

        apothecary_medication::ActiveModel {
            apothecary_id: Set(apothecary_id),
            medication_id: Set(medication_id),
            medication_quantity_type: Set(apothecary_medication::QuantityType::Package),
            medication_quantity: Set(Some(10)),
            medication_price: Set(Decimal::new(1099, 2)),
        }
        .insert(db)
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table_from_entity!(manager, apothecary_medication);
        drop_table_from_entity!(manager, medication);
        drop_table_from_entity!(manager, apothecary_user);
        drop_table_from_entity!(manager, apothecary);
        drop_table_from_entity!(manager, user);

        Ok(())
    }
}
