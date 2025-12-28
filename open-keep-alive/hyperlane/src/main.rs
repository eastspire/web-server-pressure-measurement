use hyperlane::*;
use tokio::runtime::{Builder, Runtime};

pub const BODY: &[u8] = b"Hello";

struct RootRoute;
struct PanicHook;

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

impl ServerHook for PanicHook {
    async fn new(_ctx: &Context) -> Self {
        Self
    }

    async fn handle(self, _ctx: &Context) {}
}

impl ServerHook for RootRoute {
    async fn new(_ctx: &Context) -> Self {
        Self
    }

    async fn handle(self, ctx: &Context) {
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
            .set_response_body(BODY)
            .await;
        send().await;
        while let Ok(_) = ctx.http_from_stream(RequestConfig::default()).await {
            send().await;
        }
        let _ = ctx.closed().await;
    }
}

async fn run() {
    let config: ServerConfig = ServerConfig::new().await;
    config.port(60000).await.disable_nodelay().await;
    Server::from(config)
        .await
        .panic_hook::<PanicHook>()
        .await
        .route::<RootRoute>("/")
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
