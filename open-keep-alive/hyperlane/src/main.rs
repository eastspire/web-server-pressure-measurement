use hyperlane::*;
use tokio::runtime::{Builder, Runtime};

fn runtime() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(8)
        .thread_stack_size(1024)
        .max_blocking_threads(5120)
        .max_io_events_per_tick(5120)
        .enable_all()
        .build()
        .unwrap()
}

async fn request_middleware(ctx: Context) {
    let _ = ctx
        .set_response_header(CONNECTION, KEEP_ALIVE)
        .await
        .set_response_status_code(200)
        .await
        .set_response_body("Hello")
        .await
        .send()
        .await;
    let _ = ctx.flush().await;
}

async fn run() {
    let server: Server = Server::new();
    server.host("0.0.0.0").await;
    server.port(60000).await;
    server.disable_linger().await;
    server.disable_nodelay().await;
    server.error_handler(async |_: PanicInfo| {}).await;
    server.http_buffer_size(512).await;
    server.ws_buffer_size(512).await;
    server.request_middleware(request_middleware).await;
    server.run().await.unwrap();
}

fn main() {
    runtime().block_on(run());
}
