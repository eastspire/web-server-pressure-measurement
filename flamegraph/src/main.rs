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
        let cfg: RequestConfig = RequestConfig::default();
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
        while let Ok(_) = ctx.http_from_stream(cfg).await {
            send().await;
        }
        let _ = ctx.closed().await;
    }
}

#[tokio::main]
async fn main() {
    let mut request_config: RequestConfig = RequestConfig::default();
    request_config
        .set_buffer_size(KB_4)
        .set_http_read_timeout_ms(u64::MAX)
        .set_max_body_size(usize::MAX)
        .set_max_header_count(usize::MAX)
        .set_max_header_key_length(usize::MAX)
        .set_max_header_line_length(usize::MAX)
        .set_max_header_value_length(usize::MAX)
        .set_max_path_length(usize::MAX)
        .set_max_query_length(usize::MAX)
        .set_max_request_line_length(usize::MAX)
        .set_max_ws_frame_size(usize::MAX)
        .set_max_ws_frames(usize::MAX)
        .set_ws_read_timeout_ms(u64::MAX);
    let config: ServerConfig = ServerConfig::new().await;
    config.request_config(request_config).await;
    config.port(60000).await.disable_nodelay().await;
    let server_hook: ServerControlHook = Server::from(config)
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
