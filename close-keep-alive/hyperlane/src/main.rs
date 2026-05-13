use hyperlane::*;

pub const BODY: &[u8] = b"Hello";

struct TaskPanicHook;

impl ServerHook for TaskPanicHook {
    async fn new(_: &mut Stream, _: &mut Context) -> Self {
        Self
    }

    async fn handle(self, stream: &mut Stream, ctx: &mut Context) -> Status {
        let data: Vec<u8> = ctx
            .get_mut_response()
            .set_version(HttpVersion::Http1_1)
            .set_status_code(500)
            .set_body("Internal Server Error")
            .build();
        if stream.try_send(data).await.is_err() {
            stream.set_closed(true);
            return Status::Reject;
        }
        Status::Continue
    }
}

struct RootRoute;

impl ServerHook for RootRoute {
    async fn new(_: &mut Stream, _: &mut Context) -> Self {
        Self
    }

    async fn handle(self, stream: &mut Stream, ctx: &mut Context) -> Status {
        let data: Vec<u8> = ctx
            .get_mut_response()
            .set_version(HttpVersion::Http1_1)
            .set_header(CONNECTION, CLOSE)
            .set_status_code(200)
            .set_body(BODY)
            .build();
        if stream.try_send(data).await.is_err() {
            stream.set_closed(true);
            return Status::Reject;
        }
        stream.set_closed(true);
        Status::Continue
    }
}

#[tokio::main]
async fn main() {
    let mut server: Server = Server::default();
    let mut server_config: ServerConfig = ServerConfig::default();
    server_config
        .set_address(Server::format_bind_address(DEFAULT_HOST, 60000))
        .set_nodelay(Some(false));
    server.server_config(server_config);
    server.task_panic::<TaskPanicHook>();
    server.route::<RootRoute>("/");
    let server_control_hook: ServerControlHook = server.run().await.unwrap_or_default();
    server_control_hook.wait().await;
}
