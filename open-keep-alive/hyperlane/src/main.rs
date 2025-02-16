use hyperlane::*;

fn test_sync_middleware(arc_lock_controller_data: ArcRwLockControllerData) {
    let _ = send_response(&arc_lock_controller_data, 200, "hello");
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
