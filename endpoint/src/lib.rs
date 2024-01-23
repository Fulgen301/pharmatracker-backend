use appstate::AppState;
use axum::{
    extract::Request,
    routing::{get, post},
    Router,
};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use migration::{Migrator, MigratorTrait};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tower::Service;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

mod apothecary;
mod appstate;
mod auth;
mod heartbeat;
mod reservation;
mod user;

async fn migrate(db: &entity::DatabaseConnection, drop_all: bool) -> Result<(), migration::DbErr> {
    if drop_all {
        Migrator::refresh(db).await
    } else {
        Migrator::up(db, None).await
    }
}

fn create_router(appstate: AppState) -> Router {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/heartbeat", get(heartbeat::get))
                .route("/login", post(user::login))
                .route("/register", post(user::register))
                .route("/apothecaries", get(apothecary::get))
                .route(
                    "/apothecaries/medications",
                    get(apothecary::get_medications),
                )
                .route("/reservations", get(reservation::get))
                .route("/reservations", post(reservation::post)),
        )
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(std::time::Duration::from_secs(10)),
        ))
        .with_state(appstate)
}

pub async fn run(close_rx: watch::Receiver<()>) -> anyhow::Result<()> {
    let settings = settings::Settings::new("config")?;
    let db: migration::sea_orm::prelude::DatabaseConnection =
        entity::create_database_connection(&settings.database.url).await?;

    let server_url = format!("{}:{}", settings.endpoint.host, settings.endpoint.port);

    let appstate = AppState::new(settings, db)?;

    migrate(&appstate.conn, true).await?;

    let app = create_router(appstate);

    let listener = TcpListener::bind(server_url).await?;

    let (task_close_tx, task_close_rx) = watch::channel(());

    loop {
        let mut close_rx = close_rx.clone();

        let (socket, remote_addr) = tokio::select! {
            result = listener.accept() => {
                result?
            }
            _ = close_rx.changed() => {
                info!("Shutting down server...");
                break;
            }
        };

        debug!("Accepted connection from: {}", remote_addr);

        let service = app.clone();

        let task_close_rx = task_close_rx.clone();

        tokio::spawn(async move {
            let socket = TokioIo::new(socket);

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                service.clone().call(request)
            });

            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, hyper_service)
                .with_upgrades();

            let mut conn = std::pin::pin!(conn);

            loop {
                tokio::select! {
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            debug!("Failed to serve connection: {err:#}");
                        }
                        break;
                    }
                    _ = close_rx.changed() => {
                        info!("Shutting down server...");
                        conn.as_mut().graceful_shutdown();
                        break;
                    }
                }
            }

            debug!("Connection {remote_addr} closed");

            drop(task_close_rx);
        });
    }

    drop(task_close_rx);
    drop(listener);

    debug!(
        "Waiting for {} tasks to finish",
        task_close_tx.receiver_count()
    );
    task_close_tx.closed().await;

    Ok(())
}
