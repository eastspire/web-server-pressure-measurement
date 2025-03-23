use hyperlane::*;

async fn test_sync_middleware(controller_data: ControllerData) {
    let _ = controller_data
        .set_response_header(CONNECTION, CONNECTION_KEEP_ALIVE)
        .await
        .send_response(200, "Hello")
        .await;
}

async fn run_server() {
    let server: Server = Server::new();
    server.host("0.0.0.0").await;
    server.port(60000).await;
    server.log_dir("./logs").await;
    server.disable_inner_log().await;
    server.disable_inner_print().await;
    server.http_line_buffer_size(512).await;
    server.websocket_buffer_size(512).await;
    server.log_interval_millis(1_000_000_000).await;
    server.request_middleware(test_sync_middleware).await;
    server.listen().await;
}

#[tokio::main]
async fn main() {
    run_server().await;
}
