#[macro_use]
extern crate rocket;

use std::str::FromStr;

use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use ironcalc::base::Model as IModel;
use ironcalc::import::load_from_xlsx_bytes;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::{Client, Method};
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::futures::TryFutureExt;
use rocket::http::Status;
use rocket::serde::json::Value;
use rocket::serde::Deserialize;
use rocket::State;
use roxmltree::Document;
use serde_json::json;

#[derive(Deserialize)]
struct Config {
    nextcloud_url: String,
    username: String,
    password: String,
    app_secret: String,
}

fn mk_headers(config: &Config) -> HeaderMap<HeaderValue> {
    HeaderMap::from_iter(
        [
            ("AA-VERSION", "5.0.2"),
            ("EX-APP-ID", "ironcalc"),
            ("EX-APP-VERSION", "0.1.0"),
            (
                "AUTHORIZATION-APP-API",
                BASE64_URL_SAFE
                    .encode(format!(":{}", config.app_secret))
                    .as_str(),
            ),
        ]
        .map(|(k, v)| {
            (
                HeaderName::from_str(k).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            )
        }),
    )
}
// struct

#[get("/api/webdav/<file_id>")]
async fn get_webdav(
    config: &State<Config>,
    client: &State<Client>,
    file_id: i32,
) -> Result<Vec<u8>, Status> {
    let username = &config.username;
    let search_response = client
        .request(
            Method::from_bytes(b"SEARCH").unwrap(),
            config.nextcloud_url.to_owned() + "/remote.php/dav/",
        )
        .basic_auth(username, Some(&config.password))
        .header(CONTENT_TYPE, "application/xml")
        .body(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <d:searchrequest xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns">
                <d:basicsearch>
                    <d:select><d:prop><d:displayname/></d:prop></d:select>
                    <d:from>
                        <d:scope>
                            <d:href>/files/{username}</d:href>
                            <d:depth>infinity</d:depth>
                        </d:scope>
                    </d:from>
                    <d:where>
                        <d:eq>
                            <d:prop><oc:fileid/></d:prop>
                            <d:literal>{file_id}</d:literal>
                        </d:eq>
                    </d:where>
                    <d:orderby/>
                </d:basicsearch>
            </d:searchrequest>"#
        ))
        .send()
        .and_then(|r| r.text())
        .await
        .inspect(|resp| rocket::debug!("Response: {resp:?}"))
        .map_err(|err| {
            rocket::error!("Error searching by file id: {err}");
            Status::InternalServerError
        })?;

    let search_results = Document::parse(search_response.as_str())
        .inspect(|resp| rocket::debug!("Response: {resp:?}"))
        .map_err(|err| {
            rocket::error!("Error parsing search results: {err}");
            Status::InternalServerError
        })?;

    let xlsx_path = search_results
        .descendants()
        .find(|n| n.tag_name().name() == "href")
        .and_then(|n| n.text())
        .ok_or(Status::NotFound)?;

    let xlsx_displayname = search_results
        .descendants()
        .find(|n| n.tag_name().name() == "displayname")
        .and_then(|n| n.text())
        .ok_or(Status::NotFound)?;

    let xlsx_bytes = client
        .get(config.nextcloud_url.to_owned() + xlsx_path)
        .basic_auth(username, Some(&config.password))
        .send()
        .and_then(|r| r.bytes())
        .await
        .inspect(|resp| rocket::debug!("Response: {resp:?}"))
        .map_err(|err| {
            rocket::error!("Error downloading XLSX file: {err}");
            Status::InternalServerError
        })?;

    let workbook = load_from_xlsx_bytes(
        &xlsx_bytes,
        xlsx_displayname.trim_end_matches(".xlsx"),
        "en",
        "UTC",
    )
    .inspect(|resp| rocket::debug!("Response: {resp:?}"))
    .map_err(|err| {
        rocket::error!("Error loading IronCalc worksheet: {err}");
        Status::InternalServerError
    })?;

    let model = IModel::from_workbook(workbook).map_err(|err| {
        rocket::error!("Error loading IronCalc model: {err}");
        Status::InternalServerError
    })?;

    Ok(model.to_bytes())
}

#[get("/heartbeat")]
fn heartbeat() -> Value {
    json!({ "status": "ok" })
}

#[put("/enabled?<enabled>")]
async fn enabled(
    config: &State<Config>,
    client: &State<Client>,
    enabled: i32,
) -> Result<(), Status> {
    if enabled == 1 {
        client
            .post(config.nextcloud_url.to_owned() + "/ocs/v2.php/apps/app_api/api/v1/ui/top-menu")
            .headers(mk_headers(&config))
            .json(&json!({
              "name": "ironcalc",
              "displayName": "IronCalc",
              "icon": "assets/ironcalc.svg",
              "adminRequired": "0"
            }))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .inspect(|resp| rocket::debug!("Response: {resp:?}"))
            .map_err(|err| {
                rocket::error!("Error creating top-menu: {err}");
                Status::InternalServerError
            })?;
        client
            .post(config.nextcloud_url.to_owned() + "/ocs/v2.php/apps/app_api/api/v1/ui/script")
            .headers(mk_headers(&config))
            .json(&json!({
              "type": "top_menu",
              "name": "ironcalc",
              "path": "assets/dev"
            }))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .inspect(|resp| rocket::debug!("Response: {resp:?}"))
            .map_err(|err| {
                rocket::error!("Error adding script: {err}");
                Status::InternalServerError
            })?;
        client
            .post(
                config.nextcloud_url.to_owned()
                    + "/ocs/v2.php/apps/app_api/api/v2/ui/files-actions-menu?format=json",
            )
            .headers(mk_headers(&config))
            .json(&json!({
                "name": "ironcalc",
                "displayName": "Open with IronCalc",
                "icon": "assets/ironcalc.svg",
                "actionHandler": "/files_action_handler",
                "mime": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .inspect(|resp| rocket::debug!("Response: {resp:?}"))
            .map_err(|err| {
                rocket::error!("Error adding files actions menu: {err}");
                Status::InternalServerError
            })?;
    } else {
        client
            .delete(config.nextcloud_url.to_owned() + "/ocs/v2.php/apps/app_api/api/v1/ui/script")
            .headers(mk_headers(&config))
            .json(&json!({ "type": "top_menu", "name": "ironcalc" }))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .inspect(|resp| rocket::debug!("Response: {resp:?}"))
            .map_err(|err| {
                rocket::error!("Error deleting script: {err}");
                Status::InternalServerError
            })?;
        client
            .delete(config.nextcloud_url.to_owned() + "/ocs/v2.php/apps/app_api/api/v1/ui/top-menu")
            .headers(mk_headers(&config))
            .json(&json!({ "name": "ironcalc" }))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .inspect(|resp| rocket::debug!("Response: {resp:?}"))
            .map_err(|err| {
                rocket::error!("Error deleting top-menu: {err}");
                Status::InternalServerError
            })?;
        client
            .delete(
                config.nextcloud_url.to_owned()
                    + "/ocs/v2.php/apps/app_api/api/v2/ui/files-actions-menu?format=json",
            )
            .headers(mk_headers(&config))
            .json(&json!({
              "name": "ironcalc",
            }))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .inspect(|resp| rocket::debug!("Response: {resp:?}"))
            .map_err(|err| {
                rocket::error!("Error deleting files actions menu: {err}");
                Status::InternalServerError
            })?;
    }
    Ok(())
}

#[post("/files_action_handler")]
fn files_action_handler() -> Value {
    json!({ "redirect_handler": "ironcalc" })
}

#[launch]
fn rocket() -> _ {
    let client = Client::new();

    rocket::build()
        .mount("/", routes![heartbeat, enabled, get_webdav, files_action_handler])
        .attach(AdHoc::config::<Config>())
        .manage(client)
        .mount("/assets", FileServer::from("assets"))
}
