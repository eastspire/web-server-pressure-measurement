use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::{Builder, Runtime},
};

static RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: 5\r\n\
Connection: keep-alive\r\n\r\n\
Hello";

fn runtime() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(8)
        .thread_stack_size(2097152)
        .max_blocking_threads(5120)
        .max_io_events_per_tick(5120)
        .enable_all()
        .build()
        .unwrap()
}

async fn handle_client(mut stream: TcpStream) {
    let mut buffer: [u8; 512] = [0; 512];
    let mut request: Vec<u8> = Vec::new();

    loop {
        let n: usize = match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        };

        request.extend_from_slice(&buffer[..n]);

        while let Some(pos) = find_http_end(&request) {
            if let Err(e) = stream.write_all(RESPONSE).await {
                eprintln!("Error writing response: {}", e);
                return;
            }
            request.drain(..pos + 4);
        }
    }
}

fn find_http_end(request: &[u8]) -> Option<usize> {
    for i in 0..request.len().saturating_sub(3) {
        if &request[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

fn main() {
    runtime().block_on(async move {
        let listener: TcpListener = TcpListener::bind("0.0.0.0:60000").await.unwrap();
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        handle_client(stream).await;
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    });
}
