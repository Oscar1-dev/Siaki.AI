use log::{LevelFilter, Log, Metadata, Record};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::Mutex,
};

struct FileLogger {
    file: Mutex<File>,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }
    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown-time".into());
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(
                file,
                "{timestamp} level={} target={} {}",
                record.level(),
                record.target(),
                record.args()
            );
            let _ = file.flush();
        }
    }
    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

pub fn init(directory: &Path) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("saiki.log"))?;
    log::set_boxed_logger(Box::new(FileLogger {
        file: Mutex::new(file),
    }))
    .map_err(|error| io::Error::other(error.to_string()))?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}

pub fn sanitize_field(value: &str, max: usize) -> Result<String, String> {
    let clean = value.replace(['\r', '\n'], " ");
    if clean.trim().is_empty() || clean.chars().count() > max {
        return Err(format!("log field must be 1-{max} characters"));
    }
    Ok(clean)
}

pub fn redact_context(context: BTreeMap<String, Value>) -> String {
    let safe: BTreeMap<String, Value> = context
        .into_iter()
        .map(|(key, value)| {
            let lower = key.to_ascii_lowercase();
            let sensitive = ["password", "secret", "token", "key", "auth", "cookie"]
                .iter()
                .any(|word| lower.contains(word));
            (
                key,
                if sensitive {
                    Value::String("[REDACTED]".into())
                } else {
                    value
                },
            )
        })
        .collect();
    serde_json::to_string(&safe).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removes_log_forging_newlines() {
        assert_eq!(sanitize_field("one\ntwo", 20).unwrap(), "one two");
    }
    #[test]
    fn redacts_sensitive_context() {
        let input = BTreeMap::from([
            ("apiToken".into(), Value::String("value".into())),
            ("count".into(), Value::from(2)),
        ]);
        let output = redact_context(input);
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("value"));
    }
}
