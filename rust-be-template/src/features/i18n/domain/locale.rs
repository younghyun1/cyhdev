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
}

impl UiLocale {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("ko" | "ko-KR" | "ko_kr" | "ko-kr") => Self::KoKr,
            Some("en" | "en-US" | "en_us" | "en-us") => Self::EnUs,
            _ => Self::EnUs,
        }
    }

    pub const fn as_tag(self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::KoKr => "ko-KR",
        }
    }

    pub const fn language_code(self) -> i32 {
        match self {
            Self::EnUs => EN_US_LANGUAGE_CODE,
            Self::KoKr => KO_KR_LANGUAGE_CODE,
        }
    }

    pub const fn country_code(self) -> i32 {
        match self {
            Self::EnUs => EN_US_COUNTRY_CODE,
            Self::KoKr => KO_KR_COUNTRY_CODE,
        }
    }
}
