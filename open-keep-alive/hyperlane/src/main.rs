use core_affinity::CoreId;
use hyperlane::*;
use std::thread::JoinHandle;
use tokio::runtime::{Builder, Runtime};

fn runtime() -> Runtime {
    Builder::new_multi_thread()
        .thread_stack_size(2097152)
        .max_blocking_threads(5120)
        .max_io_events_per_tick(5120)
        .enable_all()
        .build()
        .unwrap()
}

async fn request_middleware(ctx: Context) {
    let _ = ctx
        .set_response_header(CONNECTION, CONNECTION_KEEP_ALIVE)
        .await
        .send_response(200, "Hello")
        .await;
    let _ = ctx.flush().await;
}

async fn run() {
    let server: Server = Server::new();
    server.host("0.0.0.0").await;
    server.port(60000).await;
    server.disable_linger().await;
    server.disable_nodelay().await;
    server.error_handle(|_: String| {}).await;
    server.http_line_buffer_size(512).await;
    server.websocket_buffer_size(512).await;
    server.request_middleware(request_middleware).await;
    server.run().await.unwrap();
}

fn main() {
    let mut ids: Vec<CoreId> = core_affinity::get_core_ids().unwrap();
    let worker = move |id: Option<core_affinity::CoreId>| {
        if let Some(id) = id {
            let _ = core_affinity::set_for_current(id);
            runtime().block_on(run());
        }
    };
    let handle: Vec<JoinHandle<()>> = core::iter::repeat_with(|| {
        let id: Option<CoreId> = ids.pop();
        std::thread::spawn(move || worker(id))
    })
    .take(15)
    .collect::<Vec<_>>();
    worker(ids.pop());
    for handle in handle {
        handle.join().unwrap();
    }
}
