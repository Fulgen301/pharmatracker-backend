use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "medication")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::apothecary::Entity> for Entity {
    fn to() -> RelationDef {
        super::apothecary_medication::Relation::Apothecary.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::apothecary_medication::Relation::Medication
                .def()
                .rev(),
        )
    }
}

impl From<Model> for dto::medication::MedicationDetail {
    fn from(medication: Model) -> Self {
        Self {
            id: medication.id,
            name: medication.name,
        }
    }
}
