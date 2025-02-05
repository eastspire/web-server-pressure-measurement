use hyperlane::*;

fn test_sync_middleware(arc_lock_controller_data: ArcRwLockControllerData) {
    let controller_data: RwLockWriteControllerData = arc_lock_controller_data.write().unwrap();
    let mut response: Response = controller_data.get_response().clone();
    let stream: ArcTcpStream = controller_data.get_stream().clone().unwrap();
    response
        .set_body("hello".into())
        .set_status_code(200)
        .set_header(CONTENT_TYPE, APPLICATION_JSON)
        .set_header(CONTENT_ENCODING, CONTENT_ENCODING_GZIP)
        .send(&stream)
        .unwrap();
}

fn run_server() {
    let mut server: Server = Server::new();
    server.host("0.0.0.0");
    server.port(60000);
    server.log_dir("./logs");
    server.middleware(test_sync_middleware);
    server.listen();
}

#[tokio::main]
async fn main() {
    run_server();
}
