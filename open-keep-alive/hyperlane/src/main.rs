use hyperlane::*;
use tokio::runtime::{Builder, Runtime};

fn runtime() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(get_thread_count() >> 1)
        .thread_stack_size(1024)
        .max_blocking_threads(5120)
        .max_io_events_per_tick(5120)
        .enable_all()
        .build()
        .unwrap()
}

async fn request_middleware(ctx: Context) {
    let _ = ctx
        .set_response_header(CONNECTION, CONNECTION_KEEP_ALIVE)
        .await
        .send_response(200, "Hello")
        .await;
    let _ = ctx.flush().await;
}

async fn run() {
    let server: Server = Server::new();
    server.host("0.0.0.0").await;
    server.port(60000).await;
    server.disable_linger().await;
    server.disable_nodelay().await;
    server.log_dir("./logs").await;
    server.disable_log().await;
    server.disable_inner_log().await;
    server.disable_inner_print().await;
    server.http_line_buffer_size(512).await;
    server.websocket_buffer_size(512).await;
    server.request_middleware(request_middleware).await;
    server.listen().await.unwrap();
}

fn main() {
    runtime().block_on(run());
}
