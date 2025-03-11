#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> String {
    "hello".to_string()
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
