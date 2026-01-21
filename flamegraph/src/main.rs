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
        let cfg: RequestConfigData = RequestConfigData::default();
        let send = || async {
            let _ = ctx.send().await;
            let _ = ctx.flush().await;
        };
        let _ = ctx
            .set_response_version(HttpVersion::Http1_1)
            .await
            .set_response_header(CONNECTION, KEEP_ALIVE)
            .await
            .set_response_status_code(200)
            .await
            .set_response_body(BODY)
            .await;
        send().await;
        while ctx.http_from_stream(&cfg).await.is_ok() {
            send().await;
        }
        let _ = ctx.closed().await;
    }
}

#[tokio::main]
async fn main() {
    let request_config: RequestConfig = RequestConfig::new().await;
    request_config
        .buffer_size(KB_4)
        .await
        .http_read_timeout_ms(u64::MAX)
        .await
        .max_body_size(usize::MAX)
        .await
        .max_header_count(usize::MAX)
        .await
        .max_header_key_length(usize::MAX)
        .await
        .max_header_line_length(usize::MAX)
        .await
        .max_header_value_length(usize::MAX)
        .await
        .max_path_length(usize::MAX)
        .await
        .max_query_length(usize::MAX)
        .await
        .max_request_line_length(usize::MAX)
        .await
        .max_ws_frame_size(usize::MAX)
        .await
        .max_ws_frames(usize::MAX)
        .await
        .ws_read_timeout_ms(u64::MAX)
        .await;
    let server_config: ServerConfig = ServerConfig::new().await;
    server_config.port(60000).await.disable_nodelay().await;
    let server_hook: ServerControlHook = Server::new()
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
        .unwrap();
    let server_hook_clone: ServerControlHook = server_hook.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        server_hook.shutdown().await;
    });
    server_hook_clone.wait().await;
}
