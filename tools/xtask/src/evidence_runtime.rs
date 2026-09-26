//! Validate the successful outcomes required from runtime report producers.

use serde_json::Value;

use crate::{TaskError, TaskResult};

pub(crate) fn validate(label: &str, contents: &str) -> TaskResult<()> {
    let report: Value = serde_json::from_str(contents).map_err(|error| {
        TaskError(format!(
            "runtime evidence `{label}` is invalid JSON: {error}"
        ))
    })?;
    let passed = match label {
        "w8-current-secret-report" | "w8-history-secret-report" => {
            report.as_array().is_some_and(Vec::is_empty)
        }
        "w8-throughput-report" => {
            report.get("schema_version").and_then(Value::as_u64) == Some(4)
                && report
                    .get("verdict")
                    .and_then(|verdict| verdict.get("passed"))
                    .and_then(Value::as_bool)
                    == Some(true)
                && report
                    .get("verdict")
                    .and_then(|verdict| verdict.get("violations"))
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
        }
        _ => {
            return Err(TaskError(format!(
                "unknown runtime evidence producer `{label}`"
            )));
        }
    };
    if passed {
        Ok(())
    } else {
        Err(TaskError(format!(
            "runtime evidence `{label}` does not record a passing result"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::validate;

    #[test]
    fn secret_reports_require_valid_json_with_no_findings() {
        for label in ["w8-current-secret-report", "w8-history-secret-report"] {
            assert!(validate(label, "[]").is_ok());
            for report in ["[", "[] trailing", "{}", "[{\"RuleID\":\"fixture\"}]"] {
                assert!(validate(label, report).is_err(), "accepted {report}");
            }
        }
    }

    #[test]
    fn throughput_requires_a_successful_threshold_verdict() {
        let passing = r#"{"schema_version":4,"verdict":{"passed":true,"violations":[]}}"#;
        assert!(validate("w8-throughput-report", passing).is_ok());
        for report in [
            "{",
            "[]",
            r#"{"schema_version":4}"#,
            r#"{"schema_version":3,"verdict":{"passed":true,"violations":[]}}"#,
            r#"{"schema_version":4,"verdict":{"passed":false,"violations":[]}}"#,
            r#"{"schema_version":4,"verdict":{"passed":true,"violations":["too slow"]}}"#,
        ] {
            assert!(
                validate("w8-throughput-report", report).is_err(),
                "accepted {report}"
            );
        }
    }
}
