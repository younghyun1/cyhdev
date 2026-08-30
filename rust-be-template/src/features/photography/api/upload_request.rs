use axum::extract::Multipart;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err}, features::photography::{domain::photograph::PhotographContext,
    service::media::PhotographUpload}, util::media::{image_upload::{has_file_extension, is_allowed_image_mime}, staged_upload::{StageUploadError, read_bounded_text_field, stage_file_field}}};

const MAX_PHOTOGRAPH_BYTES: u64 = 150 * 1024 * 1024;
const MAX_COMMENT_BYTES: usize = 64 * 1024;
const MAX_SCALAR_BYTES: usize = 128;

pub async fn read_upload(multipart: &mut Multipart, user_id: Uuid) -> HandlerResponse<PhotographUpload> {
    let mut source = None; let mut comments = None; let mut latitude = None; let mut longitude = None;
    let mut context = PhotographContext::Photography;
    while let Some(field) = multipart.next_field().await.map_err(|error| { error!(%error, %user_id, "Failed to read photograph multipart field"); code_err(CodeError::FILE_UPLOAD_ERROR, error) })? {
        let name = field.name().map(str::to_owned);
        match name.as_deref() {
            Some("file") | None => {
                if source.is_some() { return Err(code_err(CodeError::FILE_UPLOAD_ERROR, "Only one photograph may be uploaded")); }
                validate_file(field.file_name(), field.content_type())?;
                source = Some(stage_file_field(field, MAX_PHOTOGRAPH_BYTES).await.map_err(map_stage_error)?);
            }
            Some("comments") => comments = Some(read_bounded_text_field(field, MAX_COMMENT_BYTES).await.map_err(map_stage_error)?),
            Some("lat") => { let value = read_bounded_text_field(field, MAX_SCALAR_BYTES).await.map_err(map_stage_error)?; latitude = Some(parse_coordinate(&value, "latitude", -90.0, 90.0)?); }
            Some("lon") => { let value = read_bounded_text_field(field, MAX_SCALAR_BYTES).await.map_err(map_stage_error)?; longitude = Some(parse_coordinate(&value, "longitude", -180.0, 180.0)?); }
            Some("context") | Some("photograph_context") => { let value = read_bounded_text_field(field, MAX_SCALAR_BYTES).await.map_err(map_stage_error)?;
                context = PhotographContext::parse(&value).ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "Invalid photograph context"))?; }
            Some(other) => warn!(%user_id, field = other, "Ignored unexpected photograph multipart field"),
        }
    }
    let source = source.ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "Photograph is required"))?;
    let (comments, latitude, longitude) = match context {
        PhotographContext::Photography => (required_nonempty(comments, "comments")?, required(latitude, "latitude")?, required(longitude, "longitude")?),
        PhotographContext::Post => { let fallback = source.file_name.clone().unwrap_or_else(|| "post image".to_owned());
            (comments.filter(|value| !value.trim().is_empty()).unwrap_or(fallback), latitude.unwrap_or(0.0), longitude.unwrap_or(0.0)) }
    };
    Ok(PhotographUpload { source, comments, latitude, longitude, context })
}

fn validate_file(file_name: Option<&str>, content_type: Option<&str>) -> HandlerResponse<()> {
    let name = file_name.ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "Filename is required"))?;
    if !has_file_extension(name) { return Err(code_err(CodeError::FILE_UPLOAD_ERROR, "Filename extension is required")); }
    let content_type = content_type.ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "Content type is required"))?;
    if !is_allowed_image_mime(content_type) { return Err(code_err(CodeError::FILE_UPLOAD_ERROR, "Unsupported image type")); }
    Ok(())
}
fn parse_coordinate(value: &str, name: &'static str, minimum: f64, maximum: f64) -> HandlerResponse<f64> {
    let value = value.parse::<f64>().map_err(|error| code_err(CodeError::FILE_UPLOAD_ERROR, format!("Invalid {name}: {error}")))?;
    if !value.is_finite() || !(minimum..=maximum).contains(&value) { return Err(code_err(CodeError::FILE_UPLOAD_ERROR, format!("{name} must be finite and between {minimum} and {maximum}"))); }
    Ok(value)
}
fn required_nonempty(value: Option<String>, name: &'static str) -> HandlerResponse<String> { value.filter(|value| !value.trim().is_empty()).ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, format!("Missing {name}"))) }
fn required<T>(value: Option<T>, name: &'static str) -> HandlerResponse<T> { value.ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, format!("Missing {name}"))) }
fn map_stage_error(error: StageUploadError) -> CodeErrorResp { code_err(CodeError::FILE_UPLOAD_ERROR, error) }

#[cfg(test)] mod tests { use super::parse_coordinate; #[test] fn rejects_invalid_coordinates() { assert!(parse_coordinate("NaN", "latitude", -90.0, 90.0).is_err()); assert!(parse_coordinate("181", "longitude", -180.0, 180.0).is_err()); } }
