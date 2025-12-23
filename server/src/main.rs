#[macro_use]
extern crate rocket;

use ironcalc::base::Model as IModel;
use ironcalc::import::load_from_xlsx_bytes;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method};
use rocket::fairing::AdHoc;
use rocket::futures::TryFutureExt;
use rocket::http::Status;
use rocket::serde::Deserialize;
use rocket::State;
use roxmltree::Document;

#[derive(Deserialize)]
struct Config {
    nextcloud_url: String,
    username: String,
    password: String,
}

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
        .map_err(|err| {
            rocket::error!("Error searching by file id: {err}");
            Status::InternalServerError
        })?;

    let search_results = Document::parse(search_response.as_str()).map_err(|err| {
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

#[launch]
fn rocket() -> _ {
    let client = Client::new();

    rocket::build()
        .mount("/", routes![get_webdav])
        .attach(AdHoc::config::<Config>())
        .manage(client)
}
