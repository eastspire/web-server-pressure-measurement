use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

static RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: 5\r\n\
Connection: keep-alive\r\n\r\n\
Hello";

fn handle_client(mut stream: TcpStream) {
    let mut buffer: [u8; 512] = [0; 512];
    let mut request: Vec<u8> = Vec::new();

    loop {
        let n = match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        };

        request.extend_from_slice(&buffer[..n]);

        while let Some(pos) = find_http_end(&request) {
            if let Err(e) = stream.write_all(RESPONSE) {
                eprintln!("Error writing response to stream: {}", e);
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

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:60000")?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
    Ok(())
}
