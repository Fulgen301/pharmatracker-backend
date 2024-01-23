use sea_orm::{entity::prelude::*, sea_query::ValueType, TryGetable};
use time::Time;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "schedule")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub weekday: Weekday,
    pub start: Time,
    pub end: Time,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::apothecary::Entity> for Entity {
    fn to() -> RelationDef {
        super::apothecary_schedule::Relation::Apothecary.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::apothecary_schedule::Relation::Schedule.def().rev())
    }
}

impl From<Model> for dto::schedule::Schedule {
    fn from(schedule: Model) -> Self {
        Self {
            weekday: schedule.weekday.into(),
            start: schedule.start.into(),
            end: schedule.end.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weekday(time::Weekday);

impl From<Weekday> for time::Weekday {
    fn from(weekday: Weekday) -> time::Weekday {
        weekday.0
    }
}

impl From<time::Weekday> for Weekday {
    fn from(weekday: time::Weekday) -> Weekday {
        Weekday(weekday)
    }
}

impl From<Weekday> for Value {
    fn from(weekday: Weekday) -> Value {
        Value::SmallInt(Some(weekday.0.number_from_monday() as i16))
    }
}

impl TryGetable for Weekday {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let weekday = i16::try_get_by(res, index)?;

        Ok(Weekday(match weekday {
            1 => time::Weekday::Monday,
            2 => time::Weekday::Tuesday,
            3 => time::Weekday::Wednesday,
            4 => time::Weekday::Thursday,
            5 => time::Weekday::Friday,
            6 => time::Weekday::Saturday,
            7 => time::Weekday::Sunday,
            _ => {
                return Err(sea_orm::TryGetError::DbErr(DbErr::Type(format!(
                    "Invalid weekday: {}",
                    weekday
                ))))
            }
        }))
    }
}

impl ValueType for Weekday {
    fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        let weekday = <i16 as ValueType>::try_from(v)? as u8;
        Ok(Weekday(match weekday {
            1 => time::Weekday::Monday,
            2 => time::Weekday::Tuesday,
            3 => time::Weekday::Wednesday,
            4 => time::Weekday::Thursday,
            5 => time::Weekday::Friday,
            6 => time::Weekday::Saturday,
            7 => time::Weekday::Sunday,
            _ => return Err(sea_orm::sea_query::ValueTypeErr),
        }))
    }

    fn type_name() -> String {
        stringify!(Weekday).to_owned()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::TinyInt
    }

    fn column_type() -> ColumnType {
        ColumnType::SmallInteger
    }
}
