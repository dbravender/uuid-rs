//! End-to-end gRPC contract tests for the UUID service.

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tonic::transport::{Channel, Server};
use uuid::{Variant, Version};
use uuid_rs::proto::GenerateRequest;
use uuid_rs::proto::uuid_client::UuidClient;
use uuid_rs::proto::uuid_server::UuidServer;
use uuid_rs::uuid_service::{UuidGenerator, uuid_from_response};

async fn spawn_server() -> (
    SocketAddr,
    oneshot::Sender<()>,
    JoinHandle<Result<(), tonic::transport::Error>>,
) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let address = listener.local_addr().expect("read local addr");
    let stream = tokio_stream::wrappers::TcpListenerStream::new(listener);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(UuidServer::new(UuidGenerator))
            .serve_with_incoming_shutdown(stream, async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    (address, shutdown_tx, handle)
}

#[tokio::test]
async fn grpc_generate_returns_raw_uuid_v4_bytes() {
    let (address, shutdown_tx, handle) = spawn_server().await;
    let channel = Channel::from_shared(format!("http://{address}"))
        .expect("valid endpoint")
        .connect()
        .await
        .expect("connect to test server");
    let mut client = UuidClient::new(channel);

    let response = client
        .generate(GenerateRequest {})
        .await
        .expect("generate rpc succeeds")
        .into_inner();
    let uuid = uuid_from_response(&response).expect("valid UUID bytes");

    assert_eq!(response.uuid.len(), 16);
    assert_eq!(uuid.get_version(), Some(Version::Random));
    assert_eq!(uuid.get_variant(), Variant::RFC4122);

    shutdown_tx.send(()).expect("signal shutdown");
    handle
        .await
        .expect("server task joins")
        .expect("server exits cleanly");
}
