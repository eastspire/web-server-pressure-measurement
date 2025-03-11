#[macro_use]
extern crate rocket;
use rocket::{fairing::{Fairing, Info, Kind}, http::Header, Request, Response};

struct DisableKeepAlive;

#[rocket::async_trait]
impl Fairing for DisableKeepAlive {
    fn info(&self) -> Info {
        Info {
            name: "Disable Keep-Alive",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Connection", "close"));
    }
}

#[get("/")]
async fn index() -> &'static str {
    "hello"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(DisableKeepAlive)
        .configure(rocket::Config {
            log_level: rocket::config::LogLevel::Off,
            port: 60000,
            keep_alive: 0,
            ..Default::default()
        })
}
