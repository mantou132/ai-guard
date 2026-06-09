use std::{path::PathBuf, sync::Arc};

use anyhow::{Result, bail};
use chrono::{Duration, Utc};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{
    Account, AccountStatus, AccountView, ApiType, AuditReport, AuditStatus, CreateAccount,
    LogEntry, LogQuery, ReportQuery, RiskLevel, RuntimeStats, StateFile, UpdateAccount,
};

#[derive(Clone)]
pub struct Store {
    path: Arc<PathBuf>,
    inner: Arc<RwLock<StateFile>>,
    max_logs: usize,
    max_reports: usize,
}

impl Store {
    pub async fn open(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let state = match tokio::fs::read_to_string(&path).await {
            Ok(content) => serde_json::from_str(&content)?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => StateFile::default(),
            Err(err) => return Err(err.into()),
        };

        Ok(Self {
            path: Arc::new(path),
            inner: Arc::new(RwLock::new(state)),
            max_logs: 2_000,
            max_reports: 1_000,
        })
    }

    pub async fn stats(&self) -> RuntimeStats {
        let state = self.inner.read().await;
        RuntimeStats {
            accounts: state.accounts.len(),
            enabled_accounts: state
                .accounts
                .iter()
                .filter(|account| account.enabled)
                .count(),
            logs: state.logs.len(),
            reports: state.reports.len(),
        }
    }

    pub async fn list_accounts(&self) -> Vec<AccountView> {
        let state = self.inner.read().await;
        state.accounts.iter().map(AccountView::from).collect()
    }

    pub async fn get_account(&self, id: Uuid) -> Option<Account> {
        let state = self.inner.read().await;
        state
            .accounts
            .iter()
            .find(|account| account.id == id)
            .cloned()
    }

    pub async fn create_account(&self, payload: CreateAccount) -> Result<AccountView> {
        let now = Utc::now();
        let account = Account {
            id: Uuid::new_v4(),
            name: normalize_required("name", payload.name)?,
            base_url: normalize_base_url(payload.base_url)?,
            api_key: normalize_required("api_key", payload.api_key)?,
            api_type: payload.api_type,
            supported_models: normalize_models(payload.supported_models),
            priority: payload.priority,
            enabled: payload.enabled,
            status: AccountStatus::default(),
            created_at: now,
            updated_at: now,
        };
        let view = AccountView::from(&account);

        let mut state = self.inner.write().await;
        state.accounts.push(account);
        self.save_locked(&state).await?;
        Ok(view)
    }

    pub async fn update_account(&self, id: Uuid, payload: UpdateAccount) -> Result<AccountView> {
        let mut state = self.inner.write().await;
        let account = state
            .accounts
            .iter_mut()
            .find(|account| account.id == id)
            .ok_or_else(|| anyhow::anyhow!("account not found"))?;

        if let Some(name) = payload.name {
            account.name = normalize_required("name", name)?;
        }
        if let Some(base_url) = payload.base_url {
            account.base_url = normalize_base_url(base_url)?;
        }
        if let Some(api_key) = payload.api_key {
            if !api_key.trim().is_empty() {
                account.api_key = api_key.trim().to_string();
            }
        }
        if let Some(api_type) = payload.api_type {
            account.api_type = api_type;
        }
        if let Some(supported_models) = payload.supported_models {
            account.supported_models = normalize_models(supported_models);
        }
        if let Some(priority) = payload.priority {
            account.priority = priority;
        }
        if let Some(enabled) = payload.enabled {
            account.enabled = enabled;
        }
        account.status = AccountStatus::default();
        account.updated_at = Utc::now();

        let view = AccountView::from(&*account);
        self.save_locked(&state).await?;
        Ok(view)
    }

    pub async fn delete_account(&self, id: Uuid) -> Result<()> {
        let mut state = self.inner.write().await;
        let original_len = state.accounts.len();
        state.accounts.retain(|account| account.id != id);
        if state.accounts.len() == original_len {
            bail!("account not found");
        }
        self.save_locked(&state).await
    }

    pub async fn choose_account(
        &self,
        api_type: ApiType,
        model: Option<&str>,
        exclude: &[Uuid],
    ) -> Option<Account> {
        let now = Utc::now();
        let state = self.inner.read().await;
        let mut accounts = state
            .accounts
            .iter()
            .filter(|account| {
                account.enabled
                    && !exclude.contains(&account.id)
                    && account.api_type == api_type
                    && account.status.is_available(now)
                    && model_is_supported(account, model)
            })
            .cloned()
            .collect::<Vec<_>>();

        accounts.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then(left.status.failures.cmp(&right.status.failures))
                .then(left.updated_at.cmp(&right.updated_at))
        });
        accounts.into_iter().next()
    }

    pub async fn mark_success(&self, id: Uuid) -> Result<()> {
        let mut state = self.inner.write().await;
        if let Some(account) = state.accounts.iter_mut().find(|account| account.id == id) {
            account.status.failures = 0;
            account.status.disabled_until = None;
            account.status.last_error = None;
            account.status.last_success_at = Some(Utc::now());
            account.updated_at = Utc::now();
            self.save_locked(&state).await?;
        }
        Ok(())
    }

    pub async fn mark_failure(
        &self,
        id: Uuid,
        status_code: Option<u16>,
        message: String,
    ) -> Result<()> {
        let mut state = self.inner.write().await;
        if let Some(account) = state.accounts.iter_mut().find(|account| account.id == id) {
            let failures = account.status.failures.saturating_add(1);
            let backoff = failure_backoff(status_code, failures);
            account.status.failures = failures;
            account.status.disabled_until = Some(Utc::now() + backoff);
            account.status.last_error = Some(message);
            account.status.last_failure_at = Some(Utc::now());
            account.updated_at = Utc::now();
            self.save_locked(&state).await?;
        }
        Ok(())
    }

    pub async fn append_log(&self, log: LogEntry) -> Result<LogEntry> {
        let mut state = self.inner.write().await;
        state.logs.insert(0, log.clone());
        state.logs.truncate(self.max_logs);
        self.save_locked(&state).await?;
        Ok(log)
    }

    pub async fn set_log_audit(
        &self,
        log_id: Uuid,
        status: AuditStatus,
        risk_level: RiskLevel,
    ) -> Result<()> {
        let mut state = self.inner.write().await;
        if let Some(log) = state.logs.iter_mut().find(|log| log.id == log_id) {
            let clear_payload = status == AuditStatus::Skipped
                || (status == AuditStatus::Completed && risk_level == RiskLevel::Unknown);
            log.audit_status = status;
            log.risk_level = risk_level;
            if clear_payload {
                log.request_payload = Value::Null;
                log.response_payload = Value::Null;
            }
            self.save_locked(&state).await?;
        }
        Ok(())
    }

    pub async fn pending_audit_logs(&self) -> Vec<LogEntry> {
        let state = self.inner.read().await;
        state
            .logs
            .iter()
            .filter(|log| log.audit_status == AuditStatus::Queued)
            .cloned()
            .collect()
    }

    pub async fn append_report(&self, report: AuditReport) -> Result<AuditReport> {
        let mut state = self.inner.write().await;
        state.reports.insert(0, report.clone());
        state.reports.truncate(self.max_reports);
        self.save_locked(&state).await?;
        Ok(report)
    }

    pub async fn logs(&self, query: LogQuery) -> Vec<LogEntry> {
        let limit = query.limit.unwrap_or(100).min(500);
        let state = self.inner.read().await;
        state
            .logs
            .iter()
            .filter(|log| query.level.as_ref().is_none_or(|level| &log.level == level))
            .filter(|log| {
                query
                    .risk
                    .as_ref()
                    .is_none_or(|risk_level| &log.risk_level == risk_level)
            })
            .filter(|log| {
                query
                    .account_id
                    .as_ref()
                    .is_none_or(|account_id| log.account_id.as_ref() == Some(account_id))
            })
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn reports(&self, query: ReportQuery) -> Vec<AuditReport> {
        let limit = query.limit.unwrap_or(100).min(500);
        let state = self.inner.read().await;
        state
            .reports
            .iter()
            .filter(|report| {
                query
                    .risk
                    .as_ref()
                    .is_none_or(|risk_level| &report.risk_level == risk_level)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    async fn save_locked(&self, state: &StateFile) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let content = serde_json::to_vec_pretty(state)?;
        tokio::fs::write(&*self.path, content).await?;
        Ok(())
    }
}

fn normalize_required(field: &str, value: String) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{field} is required");
    }
    Ok(value.to_string())
}

fn normalize_base_url(value: String) -> Result<String> {
    let value = normalize_required("base_url", value)?;
    if !(value.starts_with("http://") || value.starts_with("https://")) {
        bail!("base_url must start with http:// or https://");
    }
    Ok(value.trim_end_matches('/').to_string())
}

fn normalize_models(models: Vec<String>) -> Vec<String> {
    models
        .into_iter()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .collect()
}

fn model_is_supported(account: &Account, model: Option<&str>) -> bool {
    let Some(model) = model else {
        return true;
    };
    account.supported_models.is_empty()
        || account
            .supported_models
            .iter()
            .any(|supported| supported == model)
}

fn failure_backoff(status_code: Option<u16>, failures: u32) -> Duration {
    let failures = i64::from(failures.clamp(1, 20));
    let seconds = match status_code {
        Some(401 | 403) => 60 * 60,
        Some(429) => 60 * failures,
        Some(code) if code >= 500 => 30 * failures,
        _ => 15 * failures,
    };
    Duration::seconds(seconds.min(60 * 60))
}
