use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(Some(1))",
    enum_name = "user_type"
)]
pub enum UserType {
    #[sea_orm(string_value = "s")]
    Admin,
    #[sea_orm(string_value = "a")]
    Apothecary,
    #[sea_orm(string_value = "c")]
    Customer,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password: String,
    pub user_type: UserType,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::apothecary::Entity> for Entity {
    fn to() -> RelationDef {
        super::apothecary_user::Relation::Apothecary.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::apothecary_user::Relation::User.def().rev())
    }
}

impl From<Model> for dto::user::User {
    fn from(user: Model) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            user_type: match user.user_type {
                UserType::Admin => dto::user::UserType::Admin,
                UserType::Apothecary => dto::user::UserType::Apothecary,
                UserType::Customer => dto::user::UserType::Customer,
            },
        }
    }
}
