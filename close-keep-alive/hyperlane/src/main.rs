use hyperlane::*;

async fn test_sync_middleware(arc_lock_controller_data: ArcRwLockControllerData) {
    let _ = arc_lock_controller_data
        .send_response_once(200, "hello")
        .await;
}

async fn run_server() {
    let mut server: Server = Server::new();
    server.host("0.0.0.0").await;
    server.port(60000).await;
    server.log_dir("./logs").await;
    server.log_interval_millis(1_000_000_000).await;
    server.middleware(test_sync_middleware).await;
    server.listen().await;
}

#[tokio::main]
async fn main() {
    run_server().await;
}
