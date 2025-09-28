use hyperlane::*;
use tokio::runtime::{Builder, Runtime};

pub const BODY: &[u8] = b"Hello";

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
        .set_response_body(BODY)
        .await
        .send()
        .await;
    let _ = ctx.flush().await;
}

async fn run() {
    let config: ServerConfig = ServerConfig::new().await;
    config
        .host("0.0.0.0")
        .await
        .port(60000)
        .await
        .disable_nodelay()
        .await
        .buffer(256)
        .await;
    Server::from(config)
        .await
        .panic_hook(async |_: Context| {})
        .await
        .route("/", root)
        .await
        .run()
        .await
        .unwrap()
        .wait()
        .await;
}

fn main() {
    runtime().block_on(run());
}
