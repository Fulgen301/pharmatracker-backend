use tokio::{signal, sync::watch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    if cfg!(feature = "tracing") {
        console_subscriber::init();
    } else {
        tracing_subscriber::fmt::init();
    }

    tracing::info!("Starting server...");

    let (close_tx, close_rx) = watch::channel(());

    tokio::spawn(async move {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        let _ = close_tx.send(());
    });

    endpoint::run(close_rx).await
}
