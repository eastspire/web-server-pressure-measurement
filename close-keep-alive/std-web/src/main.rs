use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

static RESPONSE: &[u8] = &[
    72, 84, 84, 80, 47, 49, 46, 49, 32, 50, 48, 48, 32, 79, 75, 13, 10, 67, 111, 110, 116, 101,
    110, 116, 45, 84, 121, 112, 101, 58, 32, 116, 101, 120, 116, 47, 112, 108, 97, 105, 110, 13,
    10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 53, 13, 10, 67,
    111, 110, 110, 101, 99, 116, 105, 111, 110, 58, 32, 99, 108, 111, 115, 101, 13, 10, 13, 10,
    104, 101, 108, 108, 111,
];

fn handle_client(mut stream: TcpStream) {
    let mut buffer: [u8; 512] = [0; 512];
    let mut request: Vec<u8> = Vec::new();
    loop {
        let n = match stream.read(&mut buffer) {
            Ok(0) => {
                break;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        };
        request.extend_from_slice(&buffer[..n]);
        if let Some(pos) = find_http_end(&request) {
            if let Err(e) = stream.write_all(RESPONSE) {
                eprintln!("Error writing response to stream: {}", e);
                break;
            }
            request.drain(..pos + 4);
        }
        break;
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
                handle_client(stream);
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
    Ok(())
}
