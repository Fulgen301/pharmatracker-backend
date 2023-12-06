use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "apothecary_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub apothecary_id: i32,
    pub user_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,

    #[sea_orm(
        belongs_to = "super::apothecary::Entity",
        from = "Column::ApothecaryId",
        to = "super::apothecary::Column::Id"
    )]
    Apothecary,
}

impl ActiveModelBehavior for ActiveModel {}
