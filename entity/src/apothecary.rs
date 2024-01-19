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
        Some(super::apothecary_user::Relation::Apothecary.def())
    }
}

impl Related<super::medication::Entity> for Entity {
    fn to() -> RelationDef {
        super::apothecary_medication::Relation::Medication.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::apothecary_medication::Relation::Apothecary.def())
    }
}

impl From<Model> for dto::apothecary::ApothecaryDetail {
    fn from(apothecary: Model) -> Self {
        Self {
            id: apothecary.id,
            longitude: apothecary.longitude,
            latitude: apothecary.latitude,
            name: apothecary.name,
            street: apothecary.street,
            number: apothecary.number,
            post_code: apothecary.post_code,
            city: apothecary.city,
            country: apothecary.country,
        }
    }
}
