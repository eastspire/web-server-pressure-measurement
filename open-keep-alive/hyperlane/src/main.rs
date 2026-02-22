use hyperlane::*;

pub const BODY: &[u8] = b"Hello";

struct RootRoute;
struct TaskPanicHook;

impl ServerHook for TaskPanicHook {
    async fn new(_ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, _ctx: &mut Context) {}
}

impl ServerHook for RootRoute {
    async fn new(_ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, ctx: &mut Context) {
        let _ = ctx
            .get_mut_response()
            .set_version(HttpVersion::Http1_1)
            .set_header(CONNECTION, KEEP_ALIVE)
            .set_status_code(200)
            .set_body(BODY);
        if ctx.try_send().await.is_err() {
            ctx.set_closed(true);
            return;
        }
        let _ = ctx.flush().await;
        while ctx.http_from_stream().await.is_ok() {
            if ctx.try_send().await.is_err() {
                ctx.set_closed(true);
                return;
            }
            let _ = ctx.flush().await;
        }
        ctx.set_closed(true);
    }
}

#[tokio::main]
async fn main() {
    let request_config: RequestConfig = RequestConfig::low_security();
    let mut server_config: ServerConfig = ServerConfig::default();
    server_config
        .set_address(Server::format_bind_address(DEFAULT_HOST, 60000))
        .set_nodelay(Some(false));
    Server::default()
        .server_config(server_config)
        .request_config(request_config)
        .task_panic::<TaskPanicHook>()
        .route::<RootRoute>("/")
        .run()
        .await
        .unwrap()
        .wait()
        .await;
}
