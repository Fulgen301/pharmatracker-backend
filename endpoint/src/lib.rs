use appstate::AppState;
use axum::{extract::Request, routing::get, Router};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use migration::{Migrator, MigratorTrait};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tower::Service;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

mod appstate;
mod dto;
mod heartbeat;

async fn migrate(db: &entity::DatabaseConnection, drop_all: bool) -> Result<(), migration::DbErr> {
    if drop_all {
        Migrator::refresh(db).await
    } else {
        Migrator::up(db, None).await
    }
}

pub async fn run(close_rx: watch::Receiver<()>) -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = std::env::var("HOST").expect("HOST is not set in .env file");
    let port = std::env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let appstate = AppState::new(entity::create_database_connection(db_url).await?);

    migrate(appstate.conn(), true).await?;

    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new().route("/heartbeat", get(heartbeat::get)),
        )
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(std::time::Duration::from_secs(10)),
        ))
        .with_state(appstate);

    let listener = TcpListener::bind(&server_url).await?;

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
