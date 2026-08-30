use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};

use crate::{
    dto::{
        requests::i18n::get_ui_text_bundle_request::GetUiTextBundleRequest,
        responses::{
            i18n::ui_text_bundle_response::UiTextBundleResponse, response_data::http_resp,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::i18n::domain::keys::REQUIRED_UI_TEXT_KEYS,
    features::i18n::domain::locale::{EN_US_COUNTRY_CODE, EN_US_LANGUAGE_CODE, UiLocale},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/i18n/ui-text", tag = "i18n",
    params(GetUiTextBundleRequest),
    responses(
        (status = 200, description = "UI text bundle", body = UiTextBundleResponse),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn get_ui_text_bundle(
    State(state): State<Arc<ServerState>>,
    Query(request): Query<GetUiTextBundleRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let locale = UiLocale::parse(request.locale.as_deref());
    let texts = state
        .i18n_service()
        .ui_text_bundle(
            locale.country_code(),
            locale.language_code(),
            EN_US_COUNTRY_CODE,
            EN_US_LANGUAGE_CODE,
            REQUIRED_UI_TEXT_KEYS,
        )
        .await
        .map_err(|error| code_err(CodeError::COULD_NOT_GET_I18N_BUNDLE, error))?;
    if texts.is_empty() {
        return Err(code_err(
            CodeError::COULD_NOT_GET_I18N_BUNDLE,
            "UI text cache returned no rows",
        ));
    }
    Ok(http_resp(
        UiTextBundleResponse {
            locale: locale.as_tag().to_owned(),
            fallback_locale: UiLocale::EnUs.as_tag().to_owned(),
            texts,
        },
        (),
        start,
    ))
}
