use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing_appender::rolling;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxAuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tool_name: String,
    pub action: String,
    pub target: String,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub allowed: bool,
    pub error: Option<String>,
}

pub fn init_logging() {
    let file_appender = rolling::daily("zs-note-logs", "zs-note.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zs_note=debug,info".into()),
        )
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("ZarishNote logging initialized");
}

pub fn audit_log(entry: SandboxAuditEntry) {
    let log_path = Path::new(".znrc-audit.log");
    let json = serde_json::to_string(&entry).unwrap_or_default();

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "{}", json);
    }

    tracing::info!(
        audit_tool = %entry.tool_name,
        audit_action = %entry.action,
        audit_allowed = %entry.allowed,
        audit_duration_ms = %entry.duration_ms,
        "sandbox audit entry",
    );
}
