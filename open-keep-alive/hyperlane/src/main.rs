use hyperlane::*;

async fn test_sync_middleware(controller_data: ControllerData) {
    let _ = controller_data
        .set_response_header(CONNECTION, CONNECTION_KEEP_ALIVE)
        .await
        .send_response(200, "Hello")
        .await;
}

async fn run_server() {
    let mut server: Server = Server::new();
    server.host("0.0.0.0");
    server.port(60000);
    server.log_dir("./logs");
    server.disable_inner_log();
    server.disable_inner_print();
    server.log_interval_millis(1_000_000_000);
    server.request_middleware(test_sync_middleware);
    server.listen().await;
}

#[tokio::main]
async fn main() {
    run_server().await;
}
