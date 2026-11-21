//! Invented documents of the synthetic KCNA archive snapshot.
//!
//! Every title and body is made up for the optimization campaign; nothing here
//! quotes or paraphrases a real publication. Identifiers and dates are fixed so
//! every reset writes the same corpus. The fictional towns, harbors, orchards
//! and lanterns only give search, trends, and the reader something to match.

use kcna_archive_core::types::{Language, Source};
use time::Month;

/// One invented article.
pub struct Entry {
    /// Distinguishes the document; offset from [`ID_BASE`] to form its UUID.
    pub number: u128,
    pub date: (i32, Month, u8),
    pub language: Language,
    pub source: Source,
    pub category: &'static str,
    pub title: &'static str,
    pub body: &'static str,
}

const fn entry(
    number: u128,
    date: (i32, Month, u8),
    language: Language,
    source: Source,
    category: &'static str,
    title: &'static str,
    body: &'static str,
) -> Entry {
    Entry {
        number,
        date,
        language,
        source,
        category,
        title,
        body,
    }
}

/// UUID base of the synthetic corpus; document `n` is `ID_BASE + n`.
pub const ID_BASE: u128 = 0x0199_0000_0000_7000_8000_0000_0000_0100;
/// English edition of the one translation pair; it also has a copy.
pub const ENGLISH_EDITION: u128 = 1;
/// Korean edition of the translation pair.
pub const KOREAN_EDITION: u128 = 2;
/// Translation group joining the two editions.
pub const GROUP_ID: u128 = ID_BASE + 0x100;
/// Identifier of the copy of the English edition held by a second source.
pub const COPY_ID: u128 = ID_BASE + 0x200;

use Language::{English as E, Korean as K};
use Month::{
    April, August, December, February, January, July, June, March, May, November, October,
    September,
};
use Source::{KcnaCoJp as Jp, KcnaKp as Kp, StilesArchive as St};

/// The corpus, oldest first, across eight calendar years so yearly and monthly
/// buckets are many. One row per entry keeps the table readable, so rustfmt
/// does not reflow it.
#[rustfmt::skip]
pub const ENTRIES: [Entry; 24] = [
    entry(1, (2019, March, 12), E, Kp, "ledger", "Synthetic Aldermoor lantern ledger", "Invented ledger entry for the fictional town of Aldermoor. Twelve lantern posts were counted along the synthetic harbor road."),
    entry(2, (2019, March, 12), K, Kp, "ledger", "합성 올더무어 등불 장부", "가상의 올더무어 마을에 관한 합성 장부 항목입니다. 항구 길을 따라 등불 기둥 열두 개를 세었습니다."),
    entry(3, (2019, August, 21), E, Kp, "survey", "Synthetic orchard survey", "A made-up survey of orchard rows. Apples, pears, and plums were tallied by invented volunteers."),
    entry(4, (2019, November, 2), E, Jp, "table", "Synthetic harbor tide table", "Invented tide heights for a fictional harbor. No real place or agency is described."),
    entry(5, (2020, July, 4), E, Kp, "schedule", "Synthetic lantern festival schedule", "A fictional schedule for a lantern festival with kite races and a pie contest."),
    entry(6, (2020, September, 15), K, Kp, "survey", "합성 과수원 조사", "가상의 과수원 열을 세어 본 합성 조사입니다. 사과와 배가 포함됩니다."),
    entry(7, (2020, December, 9), E, St, "notice", "Synthetic library opening notice", "Invented notice about a library that opens on a fictional Tuesday. Visitors may borrow imaginary books."),
    entry(8, (2021, April, 17), E, Kp, "bulletin", "Synthetic harbor dredging bulletin", "A made-up bulletin about dredging the harbor channel with a pretend barge."),
    entry(9, (2021, October, 20), E, Kp, "inventory", "Synthetic lantern oil inventory", "Invented inventory of lantern oil barrels stored in a fictional warehouse."),
    entry(10, (2021, November, 30), K, Kp, "bulletin", "합성 항구 준설 공지", "가상의 항구 수로 준설에 관한 합성 공지입니다."),
    entry(11, (2022, February, 14), E, Kp, "notice", "Synthetic bridge maintenance notice", "A fictional bridge receives invented repairs. Paint colors were chosen by a pretend committee."),
    entry(12, (2022, June, 25), E, Kp, "results", "Synthetic harbor kite race results", "Invented results of a kite race above the fictional harbor."),
    entry(13, (2022, October, 10), K, Kp, "notice", "합성 도서관 개관 안내", "가상의 도서관이 문을 연다는 합성 안내입니다."),
    entry(14, (2023, February, 8), E, Kp, "notice", "Synthetic lantern repair notice", "Made-up notice listing lantern repairs for an invented street."),
    entry(15, (2023, May, 19), E, Kp, "bulletin", "Synthetic orchard harvest bulletin", "A pretend harvest bulletin with imaginary crate counts."),
    entry(16, (2023, September, 3), K, Kp, "results", "합성 연날리기 대회 결과", "가상의 연날리기 대회 결과를 적은 합성 문서입니다."),
    entry(17, (2024, January, 23), E, Kp, "schedule", "Synthetic harbor winter schedule", "Invented winter ferry schedule for a fictional harbor."),
    entry(18, (2024, June, 11), E, St, "notice", "Synthetic library catalog update", "Made-up catalog changes: three imaginary titles were added."),
    entry(19, (2024, December, 5), K, Kp, "notice", "합성 다리 보수 안내", "가상의 다리 보수 작업을 알리는 합성 안내입니다."),
    entry(20, (2025, September, 30), E, Kp, "notice", "Synthetic lantern parade notice", "Fictional parade route for lantern bearers through an invented square."),
    entry(21, (2025, November, 18), E, Jp, "bulletin", "Synthetic orchard frost bulletin", "Invented frost warning for imaginary apple trees."),
    entry(22, (2026, February, 26), K, Kp, "notice", "합성 등불 행렬 안내", "가상의 등불 행렬 경로에 관한 합성 안내입니다."),
    entry(23, (2026, May, 7), E, Kp, "bulletin", "Synthetic harbor ferry bulletin", "Invented ferry timetable change at the fictional harbor."),
    entry(24, (2026, August, 14), E, Kp, "notice", "Synthetic bridge lighting notice", "Made-up plan for lighting an invented bridge with lamps."),
];
