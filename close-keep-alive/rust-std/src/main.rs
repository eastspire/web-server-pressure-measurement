use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

static RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: 5\r\n\
Connection: close\r\n\r\n\
Hello";

fn handle_client(mut stream: TcpStream) {
    let mut buffer: [u8; 512] = [0; 512];
    let mut request: Vec<u8> = Vec::new();
    let n = match stream.read(&mut buffer) {
        Ok(0) => {
            return;
        }
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error reading from stream: {}", e);
            return;
        }
    };
    request.extend_from_slice(&buffer[..n]);
    if let Some(pos) = find_http_end(&request) {
        if let Err(e) = stream.write_all(RESPONSE) {
            eprintln!("Error writing response to stream: {}", e);
            return;
        }
    }
}

fn find_http_end(request: &[u8]) -> Option<usize> {
    for i in 0..request.len() - 3 {
        if &request[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

fn main() -> io::Result<()> {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:60000")?;
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
