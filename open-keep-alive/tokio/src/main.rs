use quinn::{Endpoint, ServerConfig};
use rustls::{Certificate, PrivateKey};
use std::{fs::File, io::BufReader, sync::Arc};

static RESPONSE: &[u8] = b"HTTP/3 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: 5\r\n\
Connection: keep-alive\r\n\r\n\
Hello";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 读取 TLS 证书
    let (certs, key) = load_certs("cert.pem", "key.pem")?;
    let server_config = configure_server(certs, key)?;

    // 2. 绑定 QUIC 监听端口
    let endpoint = Endpoint::server(server_config, "0.0.0.0:60000".parse()?)?;

    println!("QUIC server running on 0.0.0.0:60000");

    // 3. 监听 QUIC 连接
    while let Some(connecting) = endpoint.accept().await {
        tokio::spawn(async move {
            match connecting.await {
                Ok(connection) => handle_connection(connection).await,
                Err(e) => eprintln!("Failed to establish connection: {}", e),
            }
        });
    }

    Ok(())
}

/// 处理 QUIC 连接
async fn handle_connection(connection: quinn::Connection) {
    println!("New QUIC connection: {}", connection.remote_address());

    while let Ok((mut send_stream, mut recv_stream)) = connection.accept_bi().await {
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            match recv_stream.read(&mut buf).await {
                Ok(Some(_)) => {
                    if let Err(e) = send_stream.write_all(RESPONSE).await {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
                Ok(None) => {}
                Err(e) => eprintln!("Failed to read stream: {}", e),
            }
        });
    }
}

/// 加载 TLS 证书
fn load_certs(
    cert_path: &str,
    key_path: &str,
) -> Result<(Vec<Certificate>, PrivateKey), Box<dyn std::error::Error>> {
    let cert_file = File::open(cert_path)?;
    let mut reader = BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut reader)?
        .into_iter()
        .map(Certificate)
        .collect();

    let key_file = File::open(key_path)?;
    let mut reader = BufReader::new(key_file);
    let key = rustls_pemfile::pkcs8_private_keys(&mut reader)?
        .into_iter()
        .map(PrivateKey)
        .next()
        .ok_or("No private key found")?;

    Ok((certs, key))
}

/// 配置 QUIC 服务器
fn configure_server(
    certs: Vec<Certificate>,
    key: PrivateKey,
) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let mut config = ServerConfig::with_single_cert(certs, key)?;
    Arc::get_mut(&mut config.transport)
        .unwrap()
        .max_concurrent_bidi_streams(10u32.into());
    Ok(config)
}
