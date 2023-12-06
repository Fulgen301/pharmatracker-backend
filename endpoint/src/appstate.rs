use entity::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
}

impl AppState {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    #[allow(unused)]
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}
