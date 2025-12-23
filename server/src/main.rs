#[macro_use]
extern crate rocket;

use std::io;

use ironcalc::base::Model as IModel;
use ironcalc::import::load_from_xlsx_bytes;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method};
use roxmltree::Document;

#[get("/api/webdav/<file_id>")]
async fn get_webdav(file_id: &str) -> io::Result<Vec<u8>> {
    let client = Client::new();
    let res = client
        .request(
            Method::from_bytes(b"SEARCH").unwrap(),
            "http://localhost:2180/remote.php/dav/",
        )
        .basic_auth("admin", Some("admin"))
        .header(CONTENT_TYPE, "application/xml")
        .body(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <d:searchrequest xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns">
                <d:basicsearch>
                    <d:select><d:prop>
                            <d:displayname/>
                        </d:prop>
                    </d:select>
                    <d:from>
                        <d:scope>
                            <d:href>/files/admin</d:href>
                            <d:depth>infinity</d:depth>
                        </d:scope>
                    </d:from>
                    <d:where>
                        <d:eq>
                            <d:prop>
                                <oc:fileid/>
                            </d:prop>
                            <d:literal>{file_id}</d:literal>
                        </d:eq>
                    </d:where>
                    <d:orderby/>
                </d:basicsearch>
            </d:searchrequest>"#
        ))
        .send()
        .await
        .unwrap();

    let response_text = res.text().await.unwrap();

    let doc = Document::parse(response_text.as_str()).unwrap();

    let path = doc
        .descendants()
        .find(|n| n.tag_name().name() == "href")
        .unwrap()
        .text()
        .unwrap();

    let displayname = doc
        .descendants()
        .find(|n| n.tag_name().name() == "displayname")
        .unwrap()
        .text()
        .unwrap();

    let xlsx_bytes = client
        .get(format!("http://localhost:2180{path}"))
        .basic_auth("admin", Some("admin"))
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .to_vec();

    let workbook = load_from_xlsx_bytes(
        &xlsx_bytes,
        displayname.trim_end_matches(".xlsx"),
        "en",
        "UTC",
    );
    let model = IModel::from_workbook(workbook.unwrap()).unwrap();

    Ok(model.to_bytes())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![get_webdav])
}
