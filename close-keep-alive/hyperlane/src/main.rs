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

async fn root(ctx: Context) {
    let _ = ctx
        .set_response_version(HttpVersion::HTTP1_1)
        .await
        .set_response_header(CONNECTION, CLOSE)
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
    Server::new()
        .host("0.0.0.0")
        .await
        .port(60000)
        .await
        .disable_linger()
        .await
        .disable_nodelay()
        .await
        .panic_hook(async |_: Context| {})
        .await
        .http_buffer(512)
        .await
        .ws_buffer(512)
        .await
        .route("/", root)
        .await
        .run()
        .await
        .unwrap();
}

fn main() {
    runtime().block_on(run());
}
