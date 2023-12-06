use entity::{apothecary, apothecary_user, user};
use sea_orm_migration::{prelude::*, sea_orm::Schema};

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

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);

        create_table_from_entity!(manager, schema, user);
        create_table_from_entity!(manager, schema, apothecary);
        create_table_from_entity!(manager, schema, apothecary_user);

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table_from_entity!(manager, apothecary_user);
        drop_table_from_entity!(manager, apothecary);
        drop_table_from_entity!(manager, user);

        Ok(())
    }
}
