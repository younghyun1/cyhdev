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
    Ar,
    HiIn,
    BnBd,
    TeIn,
    PtBr,
    IdId,
    RuRu,
    UrPk,
    ViVn,
    TlPh,
    FaIr,
    ItIt,
    PlPl,
    SvSe,
    Sh,
    RoRo,
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
            Some("ar") => Self::Ar,
            Some("hi") => Self::HiIn,
            Some("bn") => Self::BnBd,
            Some("te") => Self::TeIn,
            Some("pt") => Self::PtBr,
            Some("id" | "in") => Self::IdId,
            Some("ru") => Self::RuRu,
            Some("ur") => Self::UrPk,
            Some("vi") => Self::ViVn,
            Some("tl" | "fil") => Self::TlPh,
            Some("fa" | "prs") => Self::FaIr,
            Some("it") => Self::ItIt,
            Some("pl") => Self::PlPl,
            Some("sv") => Self::SvSe,
            // One Latin-script catalog serves the mutually intelligible standards.
            Some("sh" | "hbs" | "sr" | "hr" | "bs" | "cnr") => Self::Sh,
            Some("ro") => Self::RoRo,
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
            Self::Ar => "ar",
            Self::HiIn => "hi-IN",
            Self::BnBd => "bn-BD",
            Self::TeIn => "te-IN",
            Self::PtBr => "pt-BR",
            Self::IdId => "id-ID",
            Self::RuRu => "ru-RU",
            Self::UrPk => "ur-PK",
            Self::ViVn => "vi-VN",
            Self::TlPh => "tl-PH",
            Self::FaIr => "fa-IR",
            Self::ItIt => "it-IT",
            Self::PlPl => "pl-PL",
            Self::SvSe => "sv-SE",
            Self::Sh => "sh",
            Self::RoRo => "ro-RO",
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
            Self::Ar => 7,
            Self::HiIn => 60,
            Self::BnBd => 19,
            Self::TeIn => 155,
            Self::PtBr => 129,
            Self::IdId => 64,
            Self::RuRu => 134,
            Self::UrPk => 171,
            Self::ViVn => 174,
            Self::TlPh => 161,
            Self::FaIr => 126,
            Self::ItIt => 71,
            Self::PlPl => 127,
            Self::SvSe => 153,
            Self::Sh => 141,
            Self::RoRo => 133,
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
            Self::Ar => 682,
            Self::HiIn => 356,
            Self::BnBd => 50,
            Self::TeIn => 356,
            Self::PtBr => 76,
            Self::IdId => 360,
            Self::RuRu => 643,
            Self::UrPk => 586,
            Self::ViVn => 704,
            Self::TlPh => 608,
            Self::FaIr => 364,
            Self::ItIt => 380,
            Self::PlPl => 616,
            Self::SvSe => 752,
            Self::Sh => 688,
            Self::RoRo => 642,
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
            ("ar-EG", UiLocale::Ar),
            ("hi", UiLocale::HiIn),
            ("bn-IN", UiLocale::BnBd),
            ("te", UiLocale::TeIn),
            ("pt-PT", UiLocale::PtBr),
            ("in", UiLocale::IdId),
            ("ru-UA", UiLocale::RuRu),
            ("ur-IN", UiLocale::UrPk),
            ("vi", UiLocale::ViVn),
            ("fil-PH", UiLocale::TlPh),
            ("fa-AF", UiLocale::FaIr),
            ("it-CH", UiLocale::ItIt),
            ("pl", UiLocale::PlPl),
            ("sv-FI", UiLocale::SvSe),
            ("hr-HR", UiLocale::Sh),
            ("sr-Latn-RS", UiLocale::Sh),
            ("bs", UiLocale::Sh),
            ("ro-MD", UiLocale::RoRo),
            ("unknown", UiLocale::EnUs),
        ] {
            assert_eq!(UiLocale::parse(Some(tag)), expected);
            assert_eq!(UiLocale::parse(Some(expected.as_tag())), expected);
        }
        assert_eq!(UiLocale::parse(None), UiLocale::EnUs);
        // Hindi and Telugu share a country, so the language row must separate them.
        assert_ne!(
            UiLocale::HiIn.language_code(),
            UiLocale::TeIn.language_code()
        );
        assert_ne!(
            UiLocale::ZhHans.country_code(),
            UiLocale::ZhHant.country_code()
        );
    }
}
