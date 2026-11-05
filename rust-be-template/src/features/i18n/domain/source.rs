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
        parse_bundle(
            UiLocale::FrFr,
            include_str!("../../../../i18n/ui/fr-FR.json"),
        )?,
        parse_bundle(
            UiLocale::EsEs,
            include_str!("../../../../i18n/ui/es-ES.json"),
        )?,
        parse_bundle(
            UiLocale::ZhHans,
            include_str!("../../../../i18n/ui/zh-Hans.json"),
        )?,
        parse_bundle(
            UiLocale::ZhHant,
            include_str!("../../../../i18n/ui/zh-Hant.json"),
        )?,
        parse_bundle(
            UiLocale::JaJp,
            include_str!("../../../../i18n/ui/ja-JP.json"),
        )?,
        parse_bundle(
            UiLocale::DeDe,
            include_str!("../../../../i18n/ui/de-DE.json"),
        )?,
        parse_bundle(UiLocale::Ar, include_str!("../../../../i18n/ui/ar.json"))?,
        parse_bundle(
            UiLocale::HiIn,
            include_str!("../../../../i18n/ui/hi-IN.json"),
        )?,
        parse_bundle(
            UiLocale::BnBd,
            include_str!("../../../../i18n/ui/bn-BD.json"),
        )?,
        parse_bundle(
            UiLocale::TeIn,
            include_str!("../../../../i18n/ui/te-IN.json"),
        )?,
        parse_bundle(
            UiLocale::PtBr,
            include_str!("../../../../i18n/ui/pt-BR.json"),
        )?,
        parse_bundle(
            UiLocale::IdId,
            include_str!("../../../../i18n/ui/id-ID.json"),
        )?,
        parse_bundle(
            UiLocale::RuRu,
            include_str!("../../../../i18n/ui/ru-RU.json"),
        )?,
        parse_bundle(
            UiLocale::UrPk,
            include_str!("../../../../i18n/ui/ur-PK.json"),
        )?,
        parse_bundle(
            UiLocale::ViVn,
            include_str!("../../../../i18n/ui/vi-VN.json"),
        )?,
        parse_bundle(
            UiLocale::TlPh,
            include_str!("../../../../i18n/ui/tl-PH.json"),
        )?,
        parse_bundle(
            UiLocale::FaIr,
            include_str!("../../../../i18n/ui/fa-IR.json"),
        )?,
        parse_bundle(
            UiLocale::ItIt,
            include_str!("../../../../i18n/ui/it-IT.json"),
        )?,
        parse_bundle(
            UiLocale::PlPl,
            include_str!("../../../../i18n/ui/pl-PL.json"),
        )?,
        parse_bundle(
            UiLocale::SvSe,
            include_str!("../../../../i18n/ui/sv-SE.json"),
        )?,
        parse_bundle(UiLocale::Sh, include_str!("../../../../i18n/ui/sh.json"))?,
        parse_bundle(
            UiLocale::RoRo,
            include_str!("../../../../i18n/ui/ro-RO.json"),
        )?,
    ])
}

fn parse_bundle(locale: UiLocale, raw: &str) -> anyhow::Result<UiTextSourceBundle> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        anyhow::anyhow!("Failed to parse {} UI text JSON: {error}", locale.as_tag())
    })?;
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

#[cfg(test)]
mod tests {
    use super::source_bundles;
    use crate::features::i18n::domain::{keys::REQUIRED_UI_TEXT_KEYS, locale::UiLocale};

    #[test]
    fn locale_sources_cover_the_registry_and_translate_eu5_navigation() -> anyhow::Result<()> {
        let bundles = source_bundles()?;
        assert_eq!(bundles.len(), 24);
        for bundle in &bundles {
            assert_eq!(bundle.entries.len(), REQUIRED_UI_TEXT_KEYS.len());
            assert_eq!(
                bundle
                    .entries
                    .iter()
                    .find(|entry| entry.key == "top_bar.site_title")
                    .map(|entry| entry.content.as_str()),
                Some("Younghyun's Blog"),
            );
        }
        let translations = bundles
            .iter()
            .filter(|bundle| matches!(bundle.locale, UiLocale::EnUs | UiLocale::KoKr))
            .filter_map(|bundle| {
                bundle
                    .entries
                    .iter()
                    .find(|entry| entry.key == "top_bar.nav.eu5_locations_db")
                    .map(|entry| (bundle.locale, entry.content.as_str()))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            translations,
            [
                (UiLocale::EnUs, "EU5 Locations DB"),
                (UiLocale::KoKr, "EU5 위치 데이터베이스"),
            ]
        );
        Ok(())
    }
}
