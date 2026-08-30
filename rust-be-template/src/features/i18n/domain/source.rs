//! Typed file-backed UI text source bundles.

use std::collections::HashSet;

use serde_json::Value;

use super::locale::UiLocale;
use crate::features::i18n::domain::keys::REQUIRED_UI_TEXT_KEYS;

pub struct UiTextSourceBundle {
    pub locale: UiLocale,
    pub entries: Vec<UiTextSourceEntry>,
}

pub struct UiTextSourceEntry {
    pub key: String,
    pub content: String,
}

const EN_US_JSON: &str = include_str!("../../../../i18n/ui/en-US.json");
const KO_KR_JSON: &str = include_str!("../../../../i18n/ui/ko-KR.json");

pub fn source_bundles() -> anyhow::Result<Vec<UiTextSourceBundle>> {
    Ok(vec![
        parse_bundle(UiLocale::EnUs, EN_US_JSON)?,
        parse_bundle(UiLocale::KoKr, KO_KR_JSON)?,
    ])
}

fn parse_bundle(locale: UiLocale, raw: &str) -> anyhow::Result<UiTextSourceBundle> {
    let value: Value = serde_json::from_str(raw)
        .map_err(|error| anyhow::anyhow!("Failed to parse {} UI text JSON: {error}", locale.as_tag()))?;
    let object = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("{} UI text JSON must be an object", locale.as_tag()))?;
    let mut seen = HashSet::<&str>::with_capacity(REQUIRED_UI_TEXT_KEYS.len());
    let mut entries = Vec::with_capacity(REQUIRED_UI_TEXT_KEYS.len());
    for required_key in REQUIRED_UI_TEXT_KEYS {
        let content = object
            .get(*required_key)
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Missing or non-string UI text key {required_key} in {}",
                    locale.as_tag()
                )
            })?;
        if !seen.insert(*required_key) {
            return Err(anyhow::anyhow!(
                "Duplicate UI text key {required_key} in {}",
                locale.as_tag()
            ));
        }
        entries.push(UiTextSourceEntry {
            key: (*required_key).to_owned(),
            content: content.to_owned(),
        });
    }
    Ok(UiTextSourceBundle { locale, entries })
}
