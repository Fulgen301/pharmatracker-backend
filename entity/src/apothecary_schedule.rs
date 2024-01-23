use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "apothecary_schedule")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub apothecary_id: Uuid,
    #[sea_orm(primary_key)]
    pub schedule_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::schedule::Entity",
        from = "Column::ScheduleId",
        to = "super::schedule::Column::Id"
    )]
    Schedule,

    #[sea_orm(
        belongs_to = "super::apothecary::Entity",
        from = "Column::ApothecaryId",
        to = "super::apothecary::Column::Id"
    )]
    Apothecary,
}

impl ActiveModelBehavior for ActiveModel {}
