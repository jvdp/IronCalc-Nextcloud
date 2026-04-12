use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, RequestBuilder};
use rocket::Request;
use rocket::futures::TryFutureExt;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::Serialize;
use serde_json::json;

use crate::Config;

pub struct ExAppContext<'r> {
    pub client: &'r Client,
    pub nextcloud_url: &'r str,
    pub aa_version: &'r str,
    pub ex_app_id: &'r str,
    pub ex_app_version: &'r str,
    pub authorization_app_api: &'r str,
    #[allow(dead_code)]
    pub aa_request_id: &'r str,
    pub user_id: String,
    #[allow(dead_code)]
    pub secret: String,
}

#[derive(Debug)]
pub struct ExAppContextError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopMenu<'a> {
    pub name: &'a str,
    pub display_name: &'a str,
    pub icon: &'a str,
    pub admin_required: &'a str,
}

#[derive(Serialize)]
pub struct Script<'a> {
    pub r#type: &'a str,
    pub name: &'a str,
    pub path: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesAction<'a> {
    pub name: &'a str,
    pub display_name: &'a str,
    pub icon: &'a str,
    pub order: i32,
    pub action_handler: &'a str,
    pub mime: &'a str,
}

impl<'r> ExAppContext<'r> {
    fn request(&self, method: Method, endpoint: &str) -> RequestBuilder {
        rocket::debug!("-> {method} {endpoint}");
        self.client
            .request(method, self.nextcloud_url.to_owned() + endpoint)
            .header("AA-VERSION", self.aa_version)
            .header("EX-APP-ID", self.ex_app_id)
            .header("EX-APP-VERSION", self.ex_app_version)
            .header("AUTHORIZATION-APP-API", self.authorization_app_api)
    }

    fn webdav_path(&self, path: &str) -> (String, String) {
        let webdav_path = format!("/remote.php/dav/files/{}/{path}", self.user_id);
        let filename = path.rsplit('/').next().unwrap_or(path).to_owned();
        (webdav_path, filename)
    }

    pub async fn download_file(&self, path: &str) -> Result<(Vec<u8>, String), Status> {
        let (webdav_path, filename) = self.webdav_path(path);

        let xlsx_bytes = self
            .request(Method::GET, &webdav_path)
            .send()
            .and_then(|r| r.bytes())
            .await
            .map_err(|err| {
                rocket::error!("Error downloading XLSX file: {err}");
                Status::InternalServerError
            })?;

        Ok((xlsx_bytes.to_vec(), filename))
    }

    pub async fn upload_file(&self, path: &str, xlsx_bytes: Vec<u8>) -> Result<(), Status> {
        let (webdav_path, _) = self.webdav_path(path);

        self.request(Method::PUT, &webdav_path)
            .header(
                CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            )
            .body(xlsx_bytes)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|err| {
                rocket::error!("Error uploading XLSX file: {err}");
                Status::InternalServerError
            })?;

        Ok(())
    }

    pub async fn rename_file(&self, path: &str, new_name: &str) -> Result<(), Status> {
        let (webdav_path, _) = self.webdav_path(path);
        let dir = webdav_path
            .rsplit_once('/')
            .map(|(d, _)| d)
            .ok_or(Status::InternalServerError)?;
        let destination = format!("{}/{dir}/{new_name}", self.nextcloud_url);

        let resp = self
            .request(
                Method::from_bytes(b"MOVE").map_err(|_| Status::InternalServerError)?,
                &webdav_path,
            )
            .header("Destination", &destination)
            .header("Overwrite", "F")
            .send()
            .await
            .map_err(|err| {
                rocket::error!("Error renaming file: {err}");
                Status::InternalServerError
            })?;

        if resp.status() == reqwest::StatusCode::PRECONDITION_FAILED {
            return Err(Status::Conflict);
        }
        resp.error_for_status().map_err(|err| {
            rocket::error!("Error renaming file: {err}");
            Status::InternalServerError
        })?;

        Ok(())
    }

    async fn register(&self, url: &str, body: &impl Serialize) -> Result<(), Status> {
        self.request(Method::POST, url)
            .json(body)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|err| {
                rocket::error!("Error registering {url}: {err}");
                Status::InternalServerError
            })?;
        Ok(())
    }

    async fn unregister(&self, url: &str, name: &str) -> Result<(), Status> {
        self.request(Method::DELETE, url)
            .json(&json!({ "name": name }))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|err| {
                rocket::error!("Error unregistering {url}: {err}");
                Status::InternalServerError
            })?;
        Ok(())
    }

    pub async fn register_top_menu(&self, menu: &TopMenu<'_>) -> Result<(), Status> {
        self.register("/ocs/v2.php/apps/app_api/api/v1/ui/top-menu", menu)
            .await
    }

    pub async fn unregister_top_menu(&self, name: &str) -> Result<(), Status> {
        self.unregister("/ocs/v2.php/apps/app_api/api/v1/ui/top-menu", name)
            .await
    }

    pub async fn register_script(&self, script: &Script<'_>) -> Result<(), Status> {
        self.register("/ocs/v2.php/apps/app_api/api/v1/ui/script", script)
            .await
    }

    pub async fn unregister_script(&self, name: &str) -> Result<(), Status> {
        self.unregister("/ocs/v2.php/apps/app_api/api/v1/ui/script", name)
            .await
    }

    pub async fn register_files_action(&self, action: &FilesAction<'_>) -> Result<(), Status> {
        self.register(
            "/ocs/v2.php/apps/app_api/api/v2/ui/files-actions-menu?format=json",
            action,
        )
        .await
    }

    pub async fn unregister_files_action(&self, name: &str) -> Result<(), Status> {
        self.unregister(
            "/ocs/v2.php/apps/app_api/api/v1/ui/files-actions-menu?format=json",
            name,
        )
        .await
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
            rocket::debug!("Request from user {user_id}");
            let config = req.rocket().state::<Config>()?;
            Some(ExAppContext {
                client: req.rocket().state::<Client>()?,
                nextcloud_url: config.nextcloud_url.as_str(),
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
