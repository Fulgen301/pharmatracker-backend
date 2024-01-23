use std::fmt::Display;

use sea_orm::{DatabaseConnection, DbErr};

pub enum ReservationServiceError {
    ApothecaryNotFound,
    MedicationNotFound,
    NotEnoughAvailable,
    ReservationAlreadyExists,
    Anyhow(anyhow::Error),
}

impl From<DbErr> for ReservationServiceError {
    fn from(err: DbErr) -> Self {
        Self::Anyhow(err.into())
    }
}

impl From<anyhow::Error> for ReservationServiceError {
    fn from(err: anyhow::Error) -> Self {
        Self::Anyhow(err)
    }
}

impl Display for ReservationServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReservationServiceError::ApothecaryNotFound => write!(f, "Apothecary not found"),
            ReservationServiceError::MedicationNotFound => write!(f, "Medication not found"),
            ReservationServiceError::NotEnoughAvailable => write!(f, "Not enough available"),
            ReservationServiceError::ReservationAlreadyExists => {
                write!(f, "Reservation already exists")
            }
            ReservationServiceError::Anyhow(e) => write!(f, "{}", e),
        }
    }
}

pub struct ReservationService {
    db: DatabaseConnection,
}

impl ReservationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
