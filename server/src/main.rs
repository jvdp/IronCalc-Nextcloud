#[macro_use]
extern crate rocket;

use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use ironcalc::base::Model as IModel;
use ironcalc::import::load_from_xlsx_bytes;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, RequestBuilder};
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::futures::TryFutureExt;
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Value;
use rocket::serde::Deserialize;
use rocket::Request;
use roxmltree::Document;
use serde_json::json;

#[derive(Deserialize)]
struct Config {
    nextcloud_url: String,
}

struct ExAppContext<'r> {
    client: &'r Client,
    nextcloud_url: &'r str,
    aa_version: &'r str,
    ex_app_id: &'r str,
    ex_app_version: &'r str,
    authorization_app_api: &'r str,
    aa_request_id: &'r str,
    user_id: String,
    secret: String,
}

#[derive(Debug)]
struct ExAppContextError;

impl<'r> ExAppContext<'r> {
    fn request(&self, method: Method, endpoint: &str) -> RequestBuilder {
        self.client
            .request(method, self.nextcloud_url.to_owned() + endpoint)
            .header("AA-VERSION", self.aa_version)
            .header("EX-APP-ID", self.ex_app_id)
            .header("EX-APP-VERSION", self.ex_app_version)
            .header("AUTHORIZATION-APP-API", self.authorization_app_api)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ExAppContext<'r> {
    type Error = ExAppContextError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        (|| {
            let authorization_app_api = req.headers().get_one("authorization-app-api")?;
            let decoded_auth = BASE64_URL_SAFE.decode(authorization_app_api).ok()?;
            let decoded_auth_str = String::from_utf8(decoded_auth).ok()?;
            let (user_id, secret) = decoded_auth_str.split_once(":")?;
            Some(ExAppContext {
                client: req.rocket().state::<Client>()?,
                nextcloud_url: req.rocket().state::<Config>()?.nextcloud_url.as_str(),
                aa_version: req.headers().get_one("aa-version")?,
                ex_app_id: req.headers().get_one("ex-app-id")?,
                ex_app_version: req.headers().get_one("ex-app-version")?,
                authorization_app_api: req.headers().get_one("authorization-app-api")?,
                aa_request_id: req.headers().get_one("aa-request-id")?,
                user_id: user_id.to_owned(),
                secret: secret.to_owned(),
            })
        })()
        .map_or(
            Outcome::Error((Status::BadRequest, ExAppContextError {})),
            Outcome::Success,
        )
    }
}

#[get("/api/webdav/<file_id>")]
async fn get_webdav(ctx: ExAppContext<'_>, file_id: i32) -> Result<Vec<u8>, Status> {
    let username = ctx.user_id.to_owned();
    let search_response = ctx
        .request(Method::from_bytes(b"SEARCH").unwrap(), "/remote.php/dav/")
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

    let xlsx_bytes = ctx
        .request(Method::GET, xlsx_path)
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
async fn enabled(ctx: ExAppContext<'_>, enabled: i32) -> Result<(), Status> {
    if enabled == 1 {
        ctx.request(Method::POST, "/ocs/v2.php/apps/app_api/api/v1/ui/top-menu")
            .json(&json!({
              "name": "ironcalc",
              "displayName": "IronCalc",
              "icon": "assets/ironcalc-white.svg",
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
        ctx.request(Method::POST, "/ocs/v2.php/apps/app_api/api/v1/ui/script")
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
        ctx.request(
            Method::POST,
            "/ocs/v2.php/apps/app_api/api/v2/ui/files-actions-menu?format=json",
        )
        .json(&json!({
            "name": "ironcalc",
            "displayName": "Open with IronCalc",
            "icon": "assets/ironcalc-black.svg",
            "order": -1000,
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
        ctx.request(Method::DELETE, "/ocs/v2.php/apps/app_api/api/v1/ui/script")
            .json(&json!({ "type": "top_menu", "name": "ironcalc" }))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .inspect(|resp| rocket::debug!("Response: {resp:?}"))
            .map_err(|err| {
                rocket::error!("Error deleting script: {err}");
                Status::InternalServerError
            })?;
        ctx.request(
            Method::DELETE,
            "/ocs/v2.php/apps/app_api/api/v1/ui/top-menu",
        )
        .json(&json!({ "name": "ironcalc" }))
        .send()
        .await
        .and_then(|resp| resp.error_for_status())
        .inspect(|resp| rocket::debug!("Response: {resp:?}"))
        .map_err(|err| {
            rocket::error!("Error deleting top-menu: {err}");
            Status::InternalServerError
        })?;
        ctx.request(
            Method::DELETE,
            "/ocs/v2.php/apps/app_api/api/v1/ui/files-actions-menu?format=json",
        )
        .json(&json!({ "name": "ironcalc" }))
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
        .mount(
            "/",
            routes![heartbeat, enabled, get_webdav, files_action_handler],
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
