#[macro_use]
extern crate rocket;

static RESPONSE: [u8; 13] = [72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33];

#[get("/")]
async fn index() -> &'static [u8] {
    &RESPONSE
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .configure(rocket::Config {
            log_level: rocket::config::LogLevel::Off,
            port: 60000,
            keep_alive: 0,
            ..Default::default()
        })
}
