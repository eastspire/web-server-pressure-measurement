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
    let send = || async {
        let _ = ctx.send().await;
        let _ = ctx.flush().await;
    };
    let _ = ctx
        .set_response_version(HttpVersion::HTTP1_1)
        .await
        .set_response_header(CONNECTION, KEEP_ALIVE)
        .await
        .set_response_status_code(200)
        .await
        .set_response_body("Hello")
        .await;
    send().await;
    while let Ok(_) = ctx.http_from_stream(256).await {
        send().await;
    }
    let _ = ctx.closed().await;
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
        .http_buffer(256)
        .await
        .ws_buffer(256)
        .await;
    let server_hook: ServerHook = Server::from(config)
        .await
        .disable_http_hook("/")
        .await
        .route("/", root)
        .await
        .run()
        .await
        .unwrap();
    let server_hook_clone: ServerHook = server_hook.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        server_hook.shutdown().await;
    });
    server_hook_clone.wait().await;
}

fn main() {
    runtime().block_on(run());
}
