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

    Server::builder()
        .add_service(UuidServer::new(UuidGenerator))
        .serve(args.listen)
        .await?;

    Ok(())
}
