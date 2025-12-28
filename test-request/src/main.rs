use std::time::{Duration, Instant};

static TIMES: u128 = 10000;
static URL: &str = "http://127.0.0.1:60000/";

async fn hyper() {
    use hyper::Uri;
    use hyper::client::*;
    let mut total_times: u128 = 0;
    let mut err_times: u128 = 0;
    for _i in 0..TIMES {
        let start: Instant = Instant::now();
        let uri: Uri = URL.parse().expect("");
        let client: Client<HttpConnector> = Client::new();
        let response: Result<hyper::Response<hyper::Body>, hyper::Error> = client.get(uri).await;
        if response.is_err() {
            err_times += 1;
        }
        let duration: Duration = start.elapsed();
        total_times += duration.as_micros();
    }
    println!("hyper agv time: {} us", total_times / TIMES);
    println!("hyper error times: {err_times}");
}

fn http_request() {
    use http_request::*;
    let mut total_times: u128 = 0;
    let mut err_times: u128 = 0;
    for _i in 0..TIMES {
        let start: Instant = Instant::now();
        let mut _request_builder: BoxRequestTrait = RequestBuilder::new()
            .get(URL)
            .unredirect()
            .buffer(100)
            .http1_1_only()
            .undecode()
            .build_sync();
        let response = _request_builder.send();
        if response.is_err() {
            err_times += 1;
        }
        let duration: Duration = start.elapsed();
        total_times += duration.as_micros();
    }
    println!("http-request agv time: {} us", total_times / TIMES);
    println!("http-request error times: {err_times}");
}

/// 阿帕奇
/// http-request agv time: 300us
/// hyper agv time: 2500us
///
/// hyperlane
/// http-request agv time: 78us
/// hyper agv time: 150us
///
/// TCP失败
/// http-request agv time: 39us
/// hyper agv time: 224us
#[tokio::main]
async fn main() {
    http_request();
    hyper().await;
}
