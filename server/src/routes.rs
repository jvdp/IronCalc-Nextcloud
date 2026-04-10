use ironcalc::base::Model as IModel;
use ironcalc::export::save_xlsx_to_writer;
use ironcalc::import::load_from_xlsx_bytes;
use rocket::data::{Data, ToByteUnit};
use rocket::http::Status;
use rocket::serde::Deserialize;
use rocket::serde::json::{Json, Value};
use serde_json::json;

use crate::context::{ExAppContext, FilesAction, Script, TopMenu};

const NEW_FILE_ID: i32 = 0;

#[get("/api/workbook/<file_id>?<path>&<lang>&<tz>")]
pub async fn get_workbook(
    ctx: ExAppContext<'_>,
    file_id: i32,
    path: Option<&str>,
    lang: Option<&str>,
    tz: Option<&str>,
) -> Result<Vec<u8>, Status> {
    let lang = lang.unwrap_or("en");
    let tz = tz.unwrap_or("UTC");

    let (xlsx_bytes, filename) = ctx.download_file(file_id, path).await?;

    let workbook = load_from_xlsx_bytes(&xlsx_bytes, filename.trim_end_matches(".xlsx"), lang, tz)
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

#[put("/api/workbook/<file_id>?<path>&<lang>", data = "<data>")]
pub async fn put_workbook(
    ctx: ExAppContext<'_>,
    file_id: i32,
    path: Option<&str>,
    lang: Option<&str>,
    data: Data<'_>,
) -> Result<Value, Status> {
    let lang = lang.unwrap_or("en");

    let model_bytes = data
        .open(ctx.max_file_size_mib.mebibytes())
        .into_bytes()
        .await
        .map_err(|err| {
            rocket::error!("Error reading request body: {err}");
            Status::InternalServerError
        })?;

    let model = IModel::from_bytes(&model_bytes, lang).map_err(|err| {
        rocket::error!("Error deserializing model: {err}");
        Status::BadRequest
    })?;

    let cursor = std::io::Cursor::new(Vec::new());
    let xlsx_bytes = save_xlsx_to_writer(&model, cursor)
        .map_err(|err| {
            rocket::error!("Error exporting to XLSX: {err}");
            Status::InternalServerError
        })?
        .into_inner();

    ctx.upload_file(file_id, path, xlsx_bytes).await?;

    let resolved_id: i64 = if file_id == NEW_FILE_ID {
        ctx.lookup_file_id(file_id, path).await?
    } else {
        file_id.into()
    };
    Ok(json!({ "fileId": resolved_id }))
}

#[post("/api/workbook/<file_id>/rename?<name>")]
pub async fn rename_workbook(
    ctx: ExAppContext<'_>,
    file_id: i32,
    name: &str,
) -> Result<Value, Status> {
    if name.contains('/') {
        return Err(Status::BadRequest);
    }

    ctx.rename_file(file_id, name).await?;
    Ok(json!({ "fileId": file_id }))
}

#[get("/heartbeat")]
pub fn heartbeat() -> Value {
    json!({ "status": "ok" })
}

#[put("/enabled?<enabled>")]
pub async fn enabled(ctx: ExAppContext<'_>, enabled: i32) -> Result<(), Status> {
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
        rocket::tokio::try_join!(
            ctx.register_top_menu(&top_menu),
            ctx.register_script(&script),
            ctx.register_files_action(&files_action),
        )?;
    } else {
        rocket::tokio::try_join!(
            ctx.unregister_top_menu(top_menu.name),
            ctx.unregister_script(script.name),
            ctx.unregister_files_action(files_action.name),
        )?;
    }
    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct FileActionPayload<'r> {
    #[serde(borrow)]
    files: Vec<FileActionPayloadFile<'r>>,
}

#[derive(Deserialize, Debug)]
pub struct FileActionPayloadFile<'r> {
    name: &'r str,
    directory: &'r str,
}

#[post(
    "/files_action_handler",
    format = "application/json",
    data = "<payload>"
)]
pub fn files_action_handler(payload: Json<FileActionPayload>) -> Result<Value, Status> {
    let file = payload.files.first().ok_or(Status::BadRequest)?;
    Ok(json!({ "redirect_handler": format!("ironcalc{}/{}", file.directory, file.name) }))
}
