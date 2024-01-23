use std::sync::Arc;

use entity::DatabaseConnection;
use service::{
    apothecary::ApothecaryService, jwt::JwtService, reservation::ReservationService,
    user::UserService,
};
use settings::Settings;

#[derive(Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub settings: Arc<Settings>,
    pub apothecary_service: Arc<ApothecaryService>,
    pub jwt_service: Arc<JwtService>,
    pub user_service: Arc<UserService>,
    pub reservation_service: Arc<ReservationService>,
}

impl AppState {
    pub fn new(settings: Settings, conn: DatabaseConnection) -> anyhow::Result<Self> {
        let settings = Arc::new(settings);
        let apothecary_service = Arc::new(ApothecaryService::new(conn.clone()));
        let jwt_service = Arc::new(JwtService::new(conn.clone(), settings.clone()));
        let user_service = Arc::new(UserService::new(conn.clone()));
        let reservation_service = Arc::new(ReservationService::new(conn.clone()));

        Ok(Self {
            conn,
            settings,
            apothecary_service,
            jwt_service,
            user_service,
            reservation_service,
        })
    }
}
