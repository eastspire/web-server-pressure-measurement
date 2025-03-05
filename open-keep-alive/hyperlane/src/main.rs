use hyperlane::*;

async fn test_sync_middleware(controller_data: ControllerData) {
    let controller_data: ControllerData = controller_data.get_controller_data().await;
    let mut response: Response = controller_data.get_response().clone();
    let body: &str = "hello";
    let stream_opt: OptionArcRwLockStream = controller_data.get_stream().await;
    if stream_opt.is_none() {
        return;
    }
    let _: ResponseResult = response
        .set_body(body)
        .set_status_code(200)
        .set_header(CONNECTION, CONNECTION_KEEP_ALIVE)
        .send(&stream_opt.unwrap())
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
