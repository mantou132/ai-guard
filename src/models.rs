use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiType {
    OpenAi,
    Anthropic,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RiskLevel {
    #[default]
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    Queued,
    Completed,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AccountStatus {
    pub disabled_until: Option<DateTime<Utc>>,
    pub failures: u32,
    pub last_error: Option<String>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
}

impl AccountStatus {
    pub fn is_available(&self, now: DateTime<Utc>) -> bool {
        match self.disabled_until {
            Some(disabled_until) => disabled_until <= now,
            None => true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub api_type: ApiType,
    #[serde(default)]
    pub supported_models: Vec<String>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountView {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub api_key_preview: String,
    pub has_api_key: bool,
    pub api_type: ApiType,
    pub supported_models: Vec<String>,
    pub priority: i32,
    pub enabled: bool,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub available: bool,
}

impl From<&Account> for AccountView {
    fn from(account: &Account) -> Self {
        let now = Utc::now();
        Self {
            id: account.id,
            name: account.name.clone(),
            base_url: account.base_url.clone(),
            api_key_preview: mask_secret(&account.api_key),
            has_api_key: !account.api_key.trim().is_empty(),
            api_type: account.api_type.clone(),
            supported_models: account.supported_models.clone(),
            priority: account.priority,
            enabled: account.enabled,
            status: account.status.clone(),
            created_at: account.created_at,
            updated_at: account.updated_at,
            available: account.enabled && account.status.is_available(now),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub api_type: ApiType,
    #[serde(default)]
    pub supported_models: Vec<String>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub api_type: Option<ApiType>,
    pub supported_models: Option<Vec<String>>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub level: LogLevel,
    pub api_type: ApiType,
    pub method: String,
    pub path: String,
    pub model: Option<String>,
    pub account_id: Option<Uuid>,
    pub account_name: Option<String>,
    pub status_code: Option<u16>,
    pub latency_ms: Option<u64>,
    pub request_payload: Value,
    pub response_payload: Value,
    pub risk_level: RiskLevel,
    pub audit_status: AuditStatus,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditReport {
    pub id: Uuid,
    pub log_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub status: AuditStatus,
    pub risk_level: RiskLevel,
    pub title: String,
    pub findings: Vec<String>,
    pub raw_response: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct StateFile {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub logs: Vec<LogEntry>,
    #[serde(default)]
    pub reports: Vec<AuditReport>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LogQuery {
    pub limit: Option<usize>,
    pub level: Option<LogLevel>,
    pub risk: Option<RiskLevel>,
    pub account_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReportQuery {
    pub limit: Option<usize>,
    pub risk: Option<RiskLevel>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ConfigView {
    pub bind: String,
    pub data_dir: String,
    pub audit_enabled: bool,
    pub audit_model: String,
    pub max_body_bytes: usize,
    pub preview_bytes: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct RuntimeStats {
    pub accounts: usize,
    pub enabled_accounts: usize,
    pub logs: usize,
    pub reports: usize,
}

pub fn default_enabled() -> bool {
    true
}

fn mask_secret(secret: &str) -> String {
    let chars = secret.chars().collect::<Vec<_>>();
    match chars.len() {
        0 => String::new(),
        1..=8 => "***".to_string(),
        len => {
            let prefix = chars.iter().take(4).collect::<String>();
            let suffix = chars.iter().skip(len - 4).collect::<String>();
            format!("{prefix}...{suffix}")
        }
    }
}
