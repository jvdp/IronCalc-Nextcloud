use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, RequestBuilder};
use rocket::Request;
use rocket::futures::TryFutureExt;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::{Deserialize, Serialize};
use roxmltree::Document;
use serde_json::json;

#[derive(Deserialize)]
pub struct Config {
    pub nextcloud_url: String,
    pub max_file_size_mib: u64,
}

pub struct ExAppContext<'r> {
    pub client: &'r Client,
    pub nextcloud_url: &'r str,
    pub max_file_size_mib: u64,
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

struct ResolvedFile {
    path: String,
    display_name: String,
}

fn xml_text(doc: &Document, tag: &str) -> Result<String, Status> {
    doc.descendants()
        .find(|n| n.tag_name().name() == tag)
        .and_then(|n| n.text())
        .map(str::to_owned)
        .ok_or(Status::NotFound)
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

    async fn resolve_webdav_path(
        &self,
        file_id: i32,
        path: Option<&str>,
    ) -> Result<(String, String), Status> {
        if let Some(p) = path {
            let webdav_path = format!("/remote.php/dav/files/{}/{p}", self.user_id);
            let filename = p.rsplit('/').next().unwrap_or(p).to_owned();
            Ok((webdav_path, filename))
        } else {
            let file = self.resolve_file(file_id).await?;
            Ok((file.path, file.display_name))
        }
    }

    pub async fn download_file(
        &self,
        file_id: i32,
        path: Option<&str>,
    ) -> Result<(Vec<u8>, String), Status> {
        let (webdav_path, filename) = self.resolve_webdav_path(file_id, path).await?;

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

    pub async fn upload_file(
        &self,
        file_id: i32,
        path: Option<&str>,
        xlsx_bytes: Vec<u8>,
    ) -> Result<(), Status> {
        let (webdav_path, _) = self.resolve_webdav_path(file_id, path).await?;

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

    pub async fn lookup_file_id(&self, file_id: i32, path: Option<&str>) -> Result<i64, Status> {
        let (webdav_path, _) = self.resolve_webdav_path(file_id, path).await?;
        self.get_file_id(&webdav_path).await
    }

    pub async fn rename_file(&self, file_id: i32, new_name: &str) -> Result<(), Status> {
        let file = self.resolve_file(file_id).await?;
        let dir = file
            .path
            .rsplit_once('/')
            .map(|(d, _)| d)
            .ok_or(Status::InternalServerError)?;
        let destination = format!("{}/{dir}/{new_name}", self.nextcloud_url);

        let resp = self
            .request(
                Method::from_bytes(b"MOVE").map_err(|_| Status::InternalServerError)?,
                &file.path,
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

    async fn webdav_xml<T>(
        &self,
        method: &[u8],
        path: &str,
        body: impl Into<reqwest::Body>,
        extract: impl FnOnce(&Document) -> Result<T, Status>,
    ) -> Result<T, Status> {
        let response = self
            .request(
                Method::from_bytes(method).map_err(|_| Status::InternalServerError)?,
                path,
            )
            .header(CONTENT_TYPE, "application/xml")
            .body(body)
            .send()
            .and_then(|r| r.text())
            .await
            .map_err(|err| {
                rocket::error!("WebDAV {path}: {err}");
                Status::InternalServerError
            })?;

        rocket::debug!("<- {response}");

        let doc = Document::parse(&response).map_err(|err| {
            rocket::error!("Error parsing WebDAV response: {err}");
            Status::InternalServerError
        })?;

        extract(&doc)
    }

    async fn resolve_file(&self, file_id: i32) -> Result<ResolvedFile, Status> {
        let user_id = &self.user_id;
        self.webdav_xml(
            b"SEARCH",
            "/remote.php/dav/",
            format!(
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
            ),
            |doc| {
                let path = xml_text(doc, "href")?;
                let display_name = xml_text(doc, "displayname")?;
                Ok(ResolvedFile { path, display_name })
            },
        )
        .await
    }

    async fn get_file_id(&self, webdav_path: &str) -> Result<i64, Status> {
        self.webdav_xml(
            b"PROPFIND",
            webdav_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <d:propfind xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns">
                <d:prop><oc:fileid/></d:prop>
            </d:propfind>"#,
            |doc| {
                xml_text(doc, "fileid")?.parse::<i64>().map_err(|err| {
                    rocket::error!("Error parsing file id: {err}");
                    Status::InternalServerError
                })
            },
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
                max_file_size_mib: config.max_file_size_mib,
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
