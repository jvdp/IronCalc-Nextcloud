#[macro_use]
extern crate rocket;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use ironcalc::base::Model as IModel;
use ironcalc::import::load_from_xlsx_bytes;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, RequestBuilder};
use rocket::Request;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::futures::TryFutureExt;
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::{Json, Value};
use rocket::serde::{Deserialize, Serialize};
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
    #[allow(dead_code)]
    aa_request_id: &'r str,
    user_id: String,
    #[allow(dead_code)]
    secret: String,
}

#[derive(Debug)]
struct ExAppContextError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TopMenu<'a> {
    name: &'a str,
    display_name: &'a str,
    icon: &'a str,
    admin_required: &'a str,
}

#[derive(Serialize)]
struct Script<'a> {
    r#type: &'a str,
    name: &'a str,
    path: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FilesAction<'a> {
    name: &'a str,
    display_name: &'a str,
    icon: &'a str,
    order: i32,
    action_handler: &'a str,
    mime: &'a str,
}

struct ResolvedFile {
    path: String,
    display_name: String,
}

impl<'r> ExAppContext<'r> {
    fn request(&self, method: Method, endpoint: &str) -> RequestBuilder {
        self.client
            .request(method, self.nextcloud_url.to_owned() + endpoint)
            .header("AA-VERSION", self.aa_version)
            .header("EX-APP-ID", self.ex_app_id)
            .header("EX-APP-VERSION", self.ex_app_version)
            .header("AUTHORIZATION-APP-API", self.authorization_app_api)
    }

    async fn send_json(
        &self,
        method: Method,
        url: &str,
        body: &Value,
        label: &str,
    ) -> Result<(), Status> {
        self.request(method, url)
            .json(body)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .inspect(|resp| rocket::debug!("Response: {resp:?}"))
            .map_err(|err| {
                rocket::error!("Error {label}: {err}");
                Status::InternalServerError
            })?;
        Ok(())
    }

    async fn register_top_menu(&self, menu: &TopMenu<'_>) -> Result<(), Status> {
        self.send_json(
            Method::POST,
            "/ocs/v2.php/apps/app_api/api/v1/ui/top-menu",
            &serde_json::to_value(menu).map_err(|_| Status::InternalServerError)?,
            "creating top-menu",
        )
        .await
    }

    async fn unregister_top_menu(&self, name: &str) -> Result<(), Status> {
        self.send_json(
            Method::DELETE,
            "/ocs/v2.php/apps/app_api/api/v1/ui/top-menu",
            &json!({ "name": name }),
            "deleting top-menu",
        )
        .await
    }

    async fn register_script(&self, script: &Script<'_>) -> Result<(), Status> {
        self.send_json(
            Method::POST,
            "/ocs/v2.php/apps/app_api/api/v1/ui/script",
            &serde_json::to_value(script).map_err(|_| Status::InternalServerError)?,
            "adding script",
        )
        .await
    }

    async fn unregister_script(&self, script: &Script<'_>) -> Result<(), Status> {
        self.send_json(
            Method::DELETE,
            "/ocs/v2.php/apps/app_api/api/v1/ui/script",
            &serde_json::to_value(script).map_err(|_| Status::InternalServerError)?,
            "deleting script",
        )
        .await
    }

    async fn register_files_action(&self, action: &FilesAction<'_>) -> Result<(), Status> {
        self.send_json(
            Method::POST,
            "/ocs/v2.php/apps/app_api/api/v2/ui/files-actions-menu?format=json",
            &serde_json::to_value(action).map_err(|_| Status::InternalServerError)?,
            "adding files actions menu",
        )
        .await
    }

    async fn unregister_files_action(&self, name: &str) -> Result<(), Status> {
        self.send_json(
            Method::DELETE,
            "/ocs/v2.php/apps/app_api/api/v1/ui/files-actions-menu?format=json",
            &json!({ "name": name }),
            "deleting files actions menu",
        )
        .await
    }

    async fn resolve_file(&self, file_id: i32) -> Result<ResolvedFile, Status> {
        let user_id = &self.user_id;
        let search_response = self
            .request(
                Method::from_bytes(b"SEARCH").map_err(|_| Status::InternalServerError)?,
                "/remote.php/dav/",
            )
            .header(CONTENT_TYPE, "application/xml")
            .body(format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
                <d:searchrequest xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns">
                    <d:basicsearch>
                        <d:select><d:prop><d:displayname/></d:prop></d:select>
                        <d:from>
                            <d:scope>
                                <d:href>/files/{user_id}</d:href>
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

        let path = search_results
            .descendants()
            .find(|n| n.tag_name().name() == "href")
            .and_then(|n| n.text())
            .ok_or(Status::NotFound)?
            .to_owned();

        let display_name = search_results
            .descendants()
            .find(|n| n.tag_name().name() == "displayname")
            .and_then(|n| n.text())
            .ok_or(Status::NotFound)?
            .to_owned();

        Ok(ResolvedFile { path, display_name })
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

#[get("/api/webdav/<file_id>?<lang>&<tz>")]
async fn get_webdav(
    ctx: ExAppContext<'_>,
    file_id: i32,
    lang: Option<&str>,
    tz: Option<&str>,
) -> Result<Vec<u8>, Status> {
    let lang = lang.unwrap_or("en");
    let tz = tz.unwrap_or("UTC");
    let file = ctx.resolve_file(file_id).await?;

    let xlsx_bytes = ctx
        .request(Method::GET, &file.path)
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
        file.display_name.trim_end_matches(".xlsx"),
        lang,
        tz,
    )
    .inspect(|resp| rocket::debug!("Response: {resp:?}"))
    .map_err(|err| {
        rocket::error!("Error loading IronCalc worksheet: {err}");
        Status::InternalServerError
    })?;

    let model = IModel::from_workbook(workbook, lang).map_err(|err| {
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
    let top_menu = TopMenu {
        name: "ironcalc",
        display_name: "IronCalc",
        icon: "assets/ironcalc-white.svg",
        admin_required: "0",
    };
    let script = Script {
        r#type: "top_menu",
        name: "ironcalc",
        path: "assets/dev",
    };
    let files_action = FilesAction {
        name: "ironcalc",
        display_name: "Open with IronCalc",
        icon: "assets/ironcalc-black.svg",
        order: -1000,
        action_handler: "/files_action_handler",
        mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    };

    if enabled == 1 {
        ctx.register_top_menu(&top_menu).await?;
        ctx.register_script(&script).await?;
        ctx.register_files_action(&files_action).await?;
    } else {
        ctx.unregister_script(&script).await?;
        ctx.unregister_top_menu(top_menu.name).await?;
        ctx.unregister_files_action(files_action.name).await?;
    }
    Ok(())
}

#[derive(Deserialize, Debug)]
struct FileActionPayload<'r> {
    #[serde(borrow)]
    files: Vec<FileActionPayloadFile<'r>>,
}

#[derive(Deserialize, Debug)]
struct FileActionPayloadFile<'r> {
    name: &'r str,
    directory: &'r str,
}

#[post(
    "/files_action_handler",
    format = "application/json",
    data = "<payload>"
)]
fn files_action_handler(payload: Json<FileActionPayload>) -> Result<Value, Status> {
    let file = payload.files.first().ok_or(Status::BadRequest)?;
    Ok(json!({ "redirect_handler": format!("ironcalc{}/{}", file.directory, file.name) }))
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
