#[macro_use]
extern crate rocket;

mod context;
mod routes;

use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::Header;
use rocket::serde::Deserialize;

use routes::{
    enabled, files_action_handler, get_workbook, heartbeat, put_workbook, rename_workbook,
};

#[derive(Deserialize)]
pub struct Config {
    pub nextcloud_url: String,
    pub max_file_size_mib: u64,
    pub script_path: String,
}

#[launch]
fn rocket() -> _ {
    let client = Client::new();

    rocket::build()
        .mount(
            "/",
            routes![
                heartbeat,
                enabled,
                get_workbook,
                put_workbook,
                rename_workbook,
                files_action_handler
            ],
        )
        .attach(AdHoc::config::<Config>())
        .attach(AdHoc::on_response("Caching headers", |_req, res| {
            Box::pin(async move {
                res.set_header(Header::new("Cache-Control", "no-store"));
            })
        }))
        .manage(client)
        .mount("/assets", FileServer::from("assets"))
}
