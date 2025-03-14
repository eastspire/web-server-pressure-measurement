#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> String {
    "Hello".to_string()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .configure(rocket::Config {
            log_level: rocket::config::LogLevel::Off,
            port: 60000,
            keep_alive: 88888888,
            ..Default::default()
        })
}
