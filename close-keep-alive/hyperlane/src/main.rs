use hyperlane::*;

pub const BODY: &[u8] = b"Hello";

struct RootRoute;
struct TaskPanicHook;

impl ServerHook for TaskPanicHook {
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
        let _ = ctx
            .set_response_version(HttpVersion::Http1_1)
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
        ctx.closed().await;
    }
}

#[tokio::main]
async fn main() {
    let request_config: RequestConfig = RequestConfig::low_security().await;
    let server_config: ServerConfig = ServerConfig::new().await;
    server_config.port(60000).await.disable_nodelay().await;
    Server::new()
        .await
        .server_config(server_config)
        .await
        .request_config(request_config)
        .await
        .task_panic::<TaskPanicHook>()
        .await
        .route::<RootRoute>("/")
        .await
        .run()
        .await
        .unwrap()
        .wait()
        .await;
}
