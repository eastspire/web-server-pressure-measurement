use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::{Builder, Runtime},
    sync::{broadcast, Mutex},
};
use std::sync::Arc;
use std::collections::HashMap;
use std::convert::TryFrom;

// WebSocket相关常量
const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

// WebSocket操作码
enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl TryFrom<u8> for OpCode {
    type Error = ();
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(OpCode::Continuation),
            0x1 => Ok(OpCode::Text),
            0x2 => Ok(OpCode::Binary),
            0x8 => Ok(OpCode::Close),
            0x9 => Ok(OpCode::Ping),
            0xA => Ok(OpCode::Pong),
            _ => Err(()),
        }
    }
}

static RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: 5\r\n\
Connection: keep-alive\r\n\r\n\
Hello";

// 广播通道容量
const BROADCAST_CAPACITY: usize = 100;

// WebSocket连接管理器
struct WebSocketManager {
    // 广播发送器
    tx: broadcast::Sender<Vec<u8>>,
    // 连接ID计数器
    next_id: u64,
    // 活跃连接ID集合 - 不再存储TcpStream，因为它不支持Clone
    connections: std::collections::HashSet<u64>,
}

impl WebSocketManager {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            tx,
            next_id: 0,
            connections: std::collections::HashSet::new(),
        }
    }
    
    // 添加新连接
    fn add_connection(&mut self) -> (u64, broadcast::Receiver<Vec<u8>>) {
        let id = self.next_id;
        self.next_id += 1;
        self.connections.insert(id);
        (id, self.tx.subscribe())
    }
    
    // 移除连接
    fn remove_connection(&mut self, id: u64) {
        self.connections.remove(&id);
    }
    
    // 广播消息给所有连接
    fn broadcast(&self, message: Vec<u8>) -> Result<usize, broadcast::error::SendError<Vec<u8>>> {
        self.tx.send(message)
    }
}

// 简单的Base64编码实现
mod base64 {
    static BASE64_TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    pub fn encode(data: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;
        
        while i + 3 <= data.len() {
            let chunk = &data[i..i+3];
            let b1 = chunk[0] as u32;
            let b2 = chunk[1] as u32;
            let b3 = chunk[2] as u32;
            
            let n = (b1 << 16) | (b2 << 8) | b3;
            
            result.push(BASE64_TABLE[(n >> 18) as usize] as char);
            result.push(BASE64_TABLE[((n >> 12) & 0x3F) as usize] as char);
            result.push(BASE64_TABLE[((n >> 6) & 0x3F) as usize] as char);
            result.push(BASE64_TABLE[(n & 0x3F) as usize] as char);
            
            i += 3;
        }
        
        if i < data.len() {
            let remaining = data.len() - i;
            let mut n = (data[i] as u32) << 16;
            
            if remaining > 1 {
                n |= (data[i+1] as u32) << 8;
            }
            
            result.push(BASE64_TABLE[(n >> 18) as usize] as char);
            result.push(BASE64_TABLE[((n >> 12) & 0x3F) as usize] as char);
            
            if remaining > 1 {
                result.push(BASE64_TABLE[((n >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            
            result.push('=');
        }
        
        result
    }
}

// 简单的SHA1实现
mod sha1 {
    pub struct Sha1 {
        h: [u32; 5],
        block: [u8; 64],
        block_len: usize,
        total_len: u64,
    }
    
    impl Sha1 {
        pub fn new() -> Self {
            Self {
                h: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
                block: [0; 64],
                block_len: 0,
                total_len: 0,
            }
        }
        
        pub fn update(&mut self, data: &[u8]) {
            let mut i = 0;
            while i < data.len() {
                self.block[self.block_len] = data[i];
                self.block_len += 1;
                self.total_len += 1;
                
                if self.block_len == 64 {
                    self.process_block();
                    self.block_len = 0;
                }
                
                i += 1;
            }
        }
        
        pub fn finalize(&mut self) -> [u8; 20] {
            // 添加填充
            self.block[self.block_len] = 0x80;
            self.block_len += 1;
            
            if self.block_len > 56 {
                while self.block_len < 64 {
                    self.block[self.block_len] = 0;
                    self.block_len += 1;
                }
                self.process_block();
                self.block_len = 0;
            }
            
            while self.block_len < 56 {
                self.block[self.block_len] = 0;
                self.block_len += 1;
            }
            
            // 添加长度（以位为单位）
            let bit_len = self.total_len * 8;
            for i in 0..8 {
                self.block[56 + i] = ((bit_len >> (56 - i * 8)) & 0xFF) as u8;
            }
            
            self.process_block();
            
            // 转换为字节数组
            let mut result = [0; 20];
            for i in 0..5 {
                let h = self.h[i];
                result[i*4] = ((h >> 24) & 0xFF) as u8;
                result[i*4 + 1] = ((h >> 16) & 0xFF) as u8;
                result[i*4 + 2] = ((h >> 8) & 0xFF) as u8;
                result[i*4 + 3] = (h & 0xFF) as u8;
            }
            
            result
        }
        
        fn process_block(&mut self) {
            let mut w = [0u32; 80];
            
            // 准备消息调度
            for i in 0..16 {
                w[i] = ((self.block[i*4] as u32) << 24) |
                       ((self.block[i*4 + 1] as u32) << 16) |
                       ((self.block[i*4 + 2] as u32) << 8) |
                       (self.block[i*4 + 3] as u32);
            }
            
            for i in 16..80 {
                w[i] = Self::rotl(w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16], 1);
            }
            
            // 初始化工作变量
            let mut a = self.h[0];
            let mut b = self.h[1];
            let mut c = self.h[2];
            let mut d = self.h[3];
            let mut e = self.h[4];
            
            // 主循环
            for i in 0..80 {
                let f: u32;
                let k: u32;
                
                if i < 20 {
                    f = (b & c) | ((!b) & d);
                    k = 0x5A827999;
                } else if i < 40 {
                    f = b ^ c ^ d;
                    k = 0x6ED9EBA1;
                } else if i < 60 {
                    f = (b & c) | (b & d) | (c & d);
                    k = 0x8F1BBCDC;
                } else {
                    f = b ^ c ^ d;
                    k = 0xCA62C1D6;
                }
                
                let temp = Self::rotl(a, 5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
                e = d;
                d = c;
                c = Self::rotl(b, 30);
                b = a;
                a = temp;
            }
            
            // 更新哈希值
            self.h[0] = self.h[0].wrapping_add(a);
            self.h[1] = self.h[1].wrapping_add(b);
            self.h[2] = self.h[2].wrapping_add(c);
            self.h[3] = self.h[3].wrapping_add(d);
            self.h[4] = self.h[4].wrapping_add(e);
        }
        
        fn rotl(x: u32, n: u32) -> u32 {
            (x << n) | (x >> (32 - n))
        }
    }
}

// 计算WebSocket握手的Accept值
fn compute_accept(key: &str) -> String {
    let mut sha = sha1::Sha1::new();
    sha.update((key.trim().to_string() + WS_GUID).as_bytes());
    let hash = sha.finalize();
    base64::encode(&hash)
}

// 解析WebSocket帧
fn parse_websocket_frame(data: &[u8]) -> Option<(bool, OpCode, Vec<u8>)> {
    if data.len() < 2 {
        return None;
    }
    
    let fin = (data[0] & 0x80) != 0;
    let opcode = match OpCode::try_from(data[0] & 0x0F) {
        Ok(op) => op,
        Err(_) => return None,
    };
    
    let masked = (data[1] & 0x80) != 0;
    if !masked {
        // 客户端发送的消息必须有掩码
        return None;
    }
    
    let mut payload_len = (data[1] & 0x7F) as usize;
    let mut mask_offset = 2;
    
    // 处理扩展长度
    if payload_len == 126 {
        if data.len() < 4 {
            return None;
        }
        payload_len = ((data[2] as usize) << 8) | (data[3] as usize);
        mask_offset = 4;
    } else if payload_len == 127 {
        if data.len() < 10 {
            return None;
        }
        // 简化处理，只取低32位
        payload_len = ((data[6] as usize) << 24) | 
                      ((data[7] as usize) << 16) | 
                      ((data[8] as usize) << 8) | 
                      (data[9] as usize);
        mask_offset = 10;
    }
    
    // 读取掩码
    if data.len() < mask_offset + 4 {
        return None;
    }
    let mask = [data[mask_offset], data[mask_offset+1], data[mask_offset+2], data[mask_offset+3]];
    let data_offset = mask_offset + 4;
    
    // 确保有足够的数据
    if data.len() < data_offset + payload_len {
        return None;
    }
    
    // 解码数据
    let mut payload = Vec::with_capacity(payload_len);
    for i in 0..payload_len {
        payload.push(data[data_offset + i] ^ mask[i % 4]);
    }
    
    Some((fin, opcode, payload))
}

// 创建WebSocket帧
fn create_websocket_frame(fin: bool, opcode: OpCode, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    
    // 第一个字节: FIN位 + 保留位 + 操作码
    let first_byte = (if fin { 0x80 } else { 0x00 }) | (opcode as u8);
    frame.push(first_byte);
    
    // 第二个字节: 掩码位(服务器发送不需要掩码) + 长度
    let payload_len = payload.len();
    if payload_len < 126 {
        frame.push(payload_len as u8);
    } else if payload_len <= 65535 {
        frame.push(126);
        frame.push(((payload_len >> 8) & 0xFF) as u8);
        frame.push((payload_len & 0xFF) as u8);
    } else {
        frame.push(127);
        // 只使用低32位
        frame.push(0); frame.push(0); frame.push(0); frame.push(0);
        frame.push(((payload_len >> 24) & 0xFF) as u8);
        frame.push(((payload_len >> 16) & 0xFF) as u8);
        frame.push(((payload_len >> 8) & 0xFF) as u8);
        frame.push((payload_len & 0xFF) as u8);
    }
    
    // 添加载荷
    frame.extend_from_slice(payload);
    
    frame
}

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

// 处理WebSocket握手
async fn handle_websocket_handshake(stream: &mut TcpStream, request: &[u8]) -> Option<()> {
    // 解析HTTP请求头
    let request_str = std::str::from_utf8(request).ok()?;
    
    // 检查是否是WebSocket升级请求
    if !request_str.contains("Upgrade: websocket") {
        return None;
    }
    
    // 提取Sec-WebSocket-Key
    let key = request_str.lines()
        .find(|line| line.starts_with("Sec-WebSocket-Key:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim())?;
    
    // 计算响应值
    let accept = compute_accept(key);
    
    // 构建握手响应
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        accept
    );
    
    // 发送握手响应
    stream.write_all(response.as_bytes()).await.ok()?;
    
    Some(())
}

// 处理WebSocket消息
async fn handle_websocket_message(stream: &mut TcpStream, manager: Arc<Mutex<WebSocketManager>>, id: u64, mut rx: broadcast::Receiver<Vec<u8>>) {
    let mut buffer = [0u8; 1024];
    let mut message_buffer = Vec::new();
    
    loop {
        tokio::select! {
            // 从客户端读取消息
            n = stream.read(&mut buffer) => {
                match n {
                    Ok(0) => break, // 连接关闭
                    Ok(n) => {
                        message_buffer.extend_from_slice(&buffer[..n]);
                        
                        // 尝试解析WebSocket帧
                        if let Some((fin, opcode, payload)) = parse_websocket_frame(&message_buffer) {
                            // 清空已处理的数据
                            message_buffer.clear();
                            
                            match opcode {
                                OpCode::Text | OpCode::Binary => {
                                    // 广播消息给所有客户端
                                    let frame = create_websocket_frame(true, OpCode::Text, &payload);
                                    let manager_lock = manager.lock().await;
                                    let _ = manager_lock.broadcast(frame);
                                },
                                OpCode::Ping => {
                                    // 响应Ping
                                    let pong = create_websocket_frame(true, OpCode::Pong, &payload);
                                    let _ = stream.write_all(&pong).await;
                                },
                                OpCode::Close => {
                                    // 关闭连接
                                    let close = create_websocket_frame(true, OpCode::Close, &[]);
                                    let _ = stream.write_all(&close).await;
                                    break;
                                },
                                _ => {}
                            }
                        }
                    },
                    Err(_) => break,
                }
            },
            
            // 接收广播消息并发送给客户端
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if let Err(_) = stream.write_all(&msg).await {
                            break;
                        }
                    },
                    Err(_) => break,
                }
            }
        }
    }
    
    // 连接关闭，从管理器中移除
    let mut manager_lock = manager.lock().await;
    manager_lock.remove_connection(id);
}

async fn handle_client(mut stream: TcpStream, manager: Arc<Mutex<WebSocketManager>>) {
    let mut buffer: [u8; 1024] = [0; 1024];
    let mut request: Vec<u8> = Vec::new();

    // 读取初始请求
    loop {
        let n: usize = match stream.read(&mut buffer).await {
            Ok(0) => return, // 连接关闭
            Ok(n) => n,
            Err(_) => return,
        };

        request.extend_from_slice(&buffer[..n]);

        // 检查是否收到完整的HTTP请求
        if let Some(pos) = find_http_end(&request) {
            // 尝试WebSocket握手
            if let Some(_) = handle_websocket_handshake(&mut stream, &request[..pos+4]).await {
                // 握手成功，添加到连接管理器
                let (id, rx) = {
                    let mut manager_lock = manager.lock().await;
                    manager_lock.add_connection()
                };
                
                // 处理WebSocket消息
                handle_websocket_message(&mut stream, manager, id, rx).await;
                return;
            } else {
                // 普通HTTP请求，发送标准响应
                if let Err(_) = stream.write_all(RESPONSE).await {
                    return;
                }
            }
            
            // 清除已处理的请求
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
        // 创建WebSocket连接管理器
        let manager = Arc::new(Mutex::new(WebSocketManager::new()));
        
        // 绑定TCP监听器
        let listener: TcpListener = TcpListener::bind("0.0.0.0:60000").await.unwrap();
        println!("WebSocket服务器已启动，监听端口: 60000");
        
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    println!("新连接: {}", addr);
                    
                    // 克隆管理器引用传递给新任务
                    let manager_clone = Arc::clone(&manager);
                    
                    tokio::spawn(async move {
                        handle_client(stream, manager_clone).await;
                    });
                }
                Err(e) => {
                    eprintln!("接受连接失败: {}", e);
                }
            }
        }
    });
}
