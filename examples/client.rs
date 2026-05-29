//! Example client that serializes UUID bytes after receiving them.

use clap::Parser;
use uuid_rs::proto::GenerateRequest;
use uuid_rs::proto::uuid_client::UuidClient;
use uuid_rs::uuid_service::uuid_from_response;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Call uuid-rs and serialize raw UUID bytes on the client side."
)]
struct Args {
    /// gRPC endpoint to call.
    #[arg(long, env = "UUID_RS_ENDPOINT", default_value = "http://127.0.0.1:50051")]
    endpoint: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut client = UuidClient::connect(args.endpoint).await?;
    let response = client.generate(GenerateRequest {}).await?.into_inner();
    let uuid = uuid_from_response(&response)?;

    println!("{}", uuid.hyphenated());
    Ok(())
}
