use std::fmt::Display;

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(Some(1))",
    enum_name = "medication_quantity_type"
)]
pub enum QuantityType {
    #[sea_orm(string_value = "p")]
    Package,
    #[sea_orm(string_value = "u")]
    Unknown,
}

impl Display for QuantityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantityType::Package => write!(f, "package"),
            QuantityType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "apothecary_medication")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub apothecary_id: Uuid,
    pub medication_id: Uuid,
    pub medication_quantity_type: QuantityType,
    pub medication_quantity: Option<u64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::medication::Entity",
        from = "Column::MedicationId",
        to = "super::medication::Column::Id"
    )]
    Medication,

    #[sea_orm(
        belongs_to = "super::apothecary::Entity",
        from = "Column::ApothecaryId",
        to = "super::apothecary::Column::Id"
    )]
    Apothecary,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<ActiveModel> for dto::medication::MedicationSearchResult {
    fn from(apothecary_medication: ActiveModel) -> Self {
        Self {
            quantity: apothecary_medication.medication_quantity,
            aliases: vec![],
            apothecary: apothecary_medication.apothecary_id,
        }
    }
}
