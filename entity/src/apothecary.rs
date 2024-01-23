use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "apothecary")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub longitude: f32,
    pub latitude: f32,
    pub street: String,
    pub number: String,
    pub post_code: i32,
    pub city: String,
    pub country: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::apothecary_user::Relation::User.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::apothecary_user::Relation::Apothecary.def().rev())
    }
}

impl Related<super::medication::Entity> for Entity {
    fn to() -> RelationDef {
        super::apothecary_medication::Relation::Medication.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::apothecary_medication::Relation::Apothecary
                .def()
                .rev(),
        )
    }
}

impl Related<super::schedule::Entity> for Entity {
    fn to() -> RelationDef {
        super::apothecary_schedule::Relation::Schedule.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::apothecary_schedule::Relation::Apothecary.def().rev())
    }
}

pub struct ApothecaryWithSchedules((Model, Vec<super::schedule::Model>));

impl From<(Model, Vec<super::schedule::Model>)> for ApothecaryWithSchedules {
    fn from((apothecary, schedule): (Model, Vec<super::schedule::Model>)) -> Self {
        Self((apothecary, schedule))
    }
}

impl From<ApothecaryWithSchedules> for dto::apothecary::ApothecaryDetail {
    fn from(apothecary_with_schedules: ApothecaryWithSchedules) -> Self {
        let (apothecary, schedule) = apothecary_with_schedules.0;
        Self {
            id: apothecary.id,
            name: apothecary.name,
            longitude: apothecary.longitude,
            latitude: apothecary.latitude,
            street: apothecary.street,
            number: apothecary.number,
            post_code: apothecary.post_code,
            city: apothecary.city,
            country: apothecary.country,
            schedules: schedule.into_iter().map(|s| s.into()).collect(),
        }
    }
}
