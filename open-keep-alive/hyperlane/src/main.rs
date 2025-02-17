use hyperlane::*;

fn test_sync_middleware(arc_lock_controller_data: ArcRwLockControllerData) {
    let controller_data: ControllerData = get_controller_data(&arc_lock_controller_data);       
    let mut response: Response = controller_data.get_response().clone();
    let body: &str = "hello";
    let stream: ArcTcpStream = controller_data.get_stream().clone().unwrap();
    let res: ResponseResult = response
        .set_body(body)
        .set_status_code(200)
        .set_header(CONNECTION, CONNECTION_KEEP_ALIVE)
        .send(&stream);
}

fn run_server() {
    let mut server: Server = Server::new();
    server.host("0.0.0.0");
    server.port(60000);
    server.log_dir("./logs");
    server.log_interval_millis(1_000_000_000);
    server.middleware(test_sync_middleware);
    server.listen();
}

#[tokio::main]
async fn main() {
    run_server();
}
