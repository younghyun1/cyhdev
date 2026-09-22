use serde_derive::Serialize;
use utoipa::ToSchema;

pub const EN_US_COUNTRY_CODE: i32 = 840;
pub const EN_US_LANGUAGE_CODE: i32 = 41;
pub const KO_KR_COUNTRY_CODE: i32 = 410;
pub const KO_KR_LANGUAGE_CODE: i32 = 86;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ToSchema)]
pub enum UiLocale {
    EnUs,
    KoKr,
    FrFr,
    EsEs,
    ZhHans,
    ZhHant,
    JaJp,
    DeDe,
}

impl UiLocale {
    pub fn parse(value: Option<&str>) -> Self {
        let normalized = value
            .unwrap_or_default()
            .replace('_', "-")
            .to_ascii_lowercase();
        let base = normalized.split('-').next();
        match base {
            Some("ko") => Self::KoKr,
            Some("fr") => Self::FrFr,
            Some("es") => Self::EsEs,
            Some("ja") => Self::JaJp,
            Some("de") => Self::DeDe,
            Some("zh") => {
                let mut parts = normalized.split('-').skip(1);
                if parts.clone().any(|part| part == "hans") {
                    Self::ZhHans
                } else if parts.any(|part| matches!(part, "hant" | "tw" | "hk" | "mo")) {
                    Self::ZhHant
                } else {
                    Self::ZhHans
                }
            }
            _ => Self::EnUs,
        }
    }

    pub const fn as_tag(self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::KoKr => "ko-KR",
            Self::FrFr => "fr-FR",
            Self::EsEs => "es-ES",
            Self::ZhHans => "zh-Hans",
            Self::ZhHant => "zh-Hant",
            Self::JaJp => "ja-JP",
            Self::DeDe => "de-DE",
        }
    }

    pub const fn language_code(self) -> i32 {
        match self {
            Self::EnUs => EN_US_LANGUAGE_CODE,
            Self::KoKr => KO_KR_LANGUAGE_CODE,
            Self::FrFr => 48,
            Self::EsEs => 149,
            Self::ZhHans | Self::ZhHant => 30,
            Self::JaJp => 73,
            Self::DeDe => 52,
        }
    }

    pub const fn country_code(self) -> i32 {
        match self {
            Self::EnUs => EN_US_COUNTRY_CODE,
            Self::KoKr => KO_KR_COUNTRY_CODE,
            Self::FrFr => 250,
            Self::EsEs => 724,
            Self::ZhHans => 156,
            Self::ZhHant => 158,
            Self::JaJp => 392,
            Self::DeDe => 276,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UiLocale;

    #[test]
    fn language_regions_and_chinese_scripts_are_normalized() {
        for (tag, expected) in [
            ("fr-CA", UiLocale::FrFr),
            ("es-MX", UiLocale::EsEs),
            ("DE_de", UiLocale::DeDe),
            ("ja-JP", UiLocale::JaJp),
            ("zh", UiLocale::ZhHans),
            ("zh-SG", UiLocale::ZhHans),
            ("zh-TW", UiLocale::ZhHant),
            ("zh-HK", UiLocale::ZhHant),
            ("zh-Hant-CN", UiLocale::ZhHant),
            ("zh-Hans-TW", UiLocale::ZhHans),
            ("unknown", UiLocale::EnUs),
        ] {
            assert_eq!(UiLocale::parse(Some(tag)), expected);
            assert_eq!(UiLocale::parse(Some(expected.as_tag())), expected);
        }
        assert_eq!(UiLocale::parse(None), UiLocale::EnUs);
        assert_ne!(
            UiLocale::ZhHans.country_code(),
            UiLocale::ZhHant.country_code()
        );
    }
}
