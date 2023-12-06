use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_type")]
pub enum UserType {
    #[sea_orm(string_value = "apothecary")]
    Apothecary,
    #[sea_orm(string_value = "user")]
    User,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub age: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::apothecary::Entity> for Entity {
    fn to() -> RelationDef {
        super::apothecary_user::Relation::Apothecary.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::apothecary_user::Relation::User.def())
    }
}
