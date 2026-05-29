//! Server binary for the UUID gRPC service.

use std::net::SocketAddr;

use clap::Parser;
use tonic::transport::Server;
use uuid_rs::proto::uuid_server::UuidServer;
use uuid_rs::uuid_service::UuidGenerator;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Address the gRPC server listens on.
    #[arg(long, env = "UUID_RS_LISTEN_ADDR", default_value = "127.0.0.1:50051")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    eprintln!("uuid-rs listening on {}", args.listen);

    Server::builder()
        .add_service(UuidServer::new(UuidGenerator))
        .serve_with_shutdown(args.listen, shutdown_signal())
        .await?;

    Ok(())
}

/// Resolve once `ctrl-c`/SIGINT or SIGTERM is received so in-flight requests drain.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install ctrl-c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }

    eprintln!("uuid-rs shutting down");
}
