//! Serializes a result set as CSV or JSON Lines.

use crate::{
    search::{engine::SearchEngine, request::Hit},
    service::{archive::ServiceError, params::QueryParams},
};

/// Rows in one export.
pub const MAX_EXPORT_ROWS: usize = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportFormat {
    Csv,
    JsonLines,
}

impl ExportFormat {
    /// Parses the `format` request parameter; absent means CSV.
    pub fn parse(value: Option<&str>) -> Option<Self> {
        match value {
            None | Some("" | "csv") => Some(Self::Csv),
            Some("jsonl") => Some(Self::JsonLines),
            Some(_) => None,
        }
    }

    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Csv => "text/csv; charset=utf-8",
            Self::JsonLines => "application/x-ndjson",
        }
    }

    pub const fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::JsonLines => "jsonl",
        }
    }
}

/// Serializes up to [`MAX_EXPORT_ROWS`] results of a query.
pub fn export(
    engine: &SearchEngine,
    params: &QueryParams,
    format: ExportFormat,
) -> Result<String, ServiceError> {
    let filters = params.filters()?;
    let sort = params.sort()?;
    if params.is_unsearchable(&filters) {
        return Ok(String::new());
    }
    let page = engine.search(&filters, sort, 0, MAX_EXPORT_ROWS)?;
    Ok(match format {
        ExportFormat::Csv => csv(&page.hits),
        ExportFormat::JsonLines => json_lines(&page.hits),
    })
}

fn sources(hit: &Hit) -> String {
    hit.sources
        .iter()
        .map(|source| source.code())
        .collect::<Vec<_>>()
        .join(";")
}

fn csv(hits: &[Hit]) -> String {
    let mut output = String::from("id,date,language,sources,category,title\r\n");
    for hit in hits {
        let fields = [
            hit.id.to_string(),
            hit.date.to_string(),
            hit.language.code().to_owned(),
            sources(hit),
            hit.category.clone().unwrap_or_default(),
            hit.title.clone(),
        ];
        let row = fields
            .iter()
            .map(|field| csv_field(field))
            .collect::<Vec<_>>();
        output.push_str(&row.join(","));
        output.push_str("\r\n");
    }
    output
}

/// Quotes a field when it needs it, and neutralizes a leading character that
/// spreadsheet programs would evaluate as a formula.
fn csv_field(value: &str) -> String {
    let guarded = if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{value}")
    } else {
        value.to_owned()
    };
    if guarded.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", guarded.replace('"', "\"\""))
    } else {
        guarded
    }
}

fn json_lines(hits: &[Hit]) -> String {
    let mut output = String::new();
    for hit in hits {
        let line = serde_json::json!({
            "id": hit.id.to_string(),
            "date": hit.date.to_string(),
            "language": hit.language.code(),
            "sources": hit.sources.iter().map(|source| source.code()).collect::<Vec<_>>(),
            "category": hit.category,
            "title": hit.title,
            "text": hit.body,
        });
        output.push_str(&line.to_string());
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use time::macros::date;
    use uuid::Uuid;

    use super::*;
    use crate::types::{Language, Source};

    fn hit(title: &str, category: Option<&str>) -> Hit {
        Hit {
            id: Uuid::from_u128(7),
            title: title.to_owned(),
            body: Some("본문".to_owned()),
            date: date!(2026 - 11 - 21),
            language: Language::Korean,
            sources: vec![Source::KcnaKp, Source::KcnaCoJp],
            category: category.map(str::to_owned),
        }
    }

    #[test]
    fn csv_quotes_fields_and_guards_formulas() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_field("=SUM(A1)"), "'=SUM(A1)");
        assert_eq!(csv_field("-1, then"), "\"'-1, then\"");
        let output = csv(&[hit("제목, 부제", Some("국내소식"))]);
        let mut lines = output.split("\r\n");
        assert_eq!(
            lines.next(),
            Some("id,date,language,sources,category,title")
        );
        assert_eq!(
            lines.next(),
            Some(
                "00000000-0000-0000-0000-000000000007,2026-11-21,kor,kcna_kp;kcna_co_jp,국내소식,\"제목, 부제\""
            )
        );
    }

    #[test]
    fn json_lines_holds_one_object_per_row() {
        let output = json_lines(&[hit("제목", None), hit("둘째", None)]);
        let rows = output
            .lines()
            .map(serde_json::from_str::<serde_json::Value>)
            .collect::<Result<Vec<_>, _>>();
        assert!(rows.is_ok_and(|rows| {
            rows.len() == 2
                && rows[0]["title"] == "제목"
                && rows[0]["text"] == "본문"
                && rows[0]["sources"][1] == "kcna_co_jp"
                && rows[0]["category"].is_null()
        }));
    }
}
