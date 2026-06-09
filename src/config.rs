use std::path::PathBuf;

use clap::Parser;

fn default_data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("unable to determine home directory")
        .join(".ai-guard")
}

#[derive(Clone, Debug, Parser)]
#[command(
    version,
    about = "Local AI API relay, log viewer, account router, and async auditor."
)]
pub struct Config {
    #[arg(long, env = "AI_GUARD_BIND", default_value = "127.0.0.1:8787")]
    pub bind: String,

    #[arg(
        long,
        env = "AI_GUARD_DATA_DIR",
        default_value_os_t = default_data_dir().into_os_string().into()
    )]
    pub data_dir: PathBuf,

    #[arg(long, env = "AI_GUARD_OPENROUTER_KEY")]
    pub openrouter_key: Option<String>,

    #[arg(
        long,
        env = "AI_GUARD_OPENROUTER_BASE_URL",
        default_value = "https://openrouter.ai/api/v1"
    )]
    pub openrouter_base_url: String,

    #[arg(
        long,
        env = "AI_GUARD_AUDIT_MODEL",
        default_value = "nvidia/nemotron-3-ultra-550b-a55b:free"
    )]
    pub audit_model: String,

    #[arg(long, env = "AI_GUARD_MAX_BODY_BYTES", default_value_t = 4 * 1024 * 1024)]
    pub max_body_bytes: usize,

    #[arg(long, env = "AI_GUARD_PREVIEW_BYTES", default_value_t = 4096)]
    pub preview_bytes: usize,

    #[arg(long, env = "AI_GUARD_RESEND_KEY")]
    pub resend_key: Option<String>,

    #[arg(long, env = "AI_GUARD_RESEND_EMAIL")]
    pub resend_email: Option<String>,

    #[arg(
        long,
        env = "AI_GUARD_RESEND_FROM",
        default_value = "AI Guard <onboarding@resend.dev>"
    )]
    pub resend_from: String,
}

impl Config {
    pub fn state_path(&self) -> PathBuf {
        self.data_dir.join("state.json")
    }

    pub fn audit_enabled(&self) -> bool {
        self.openrouter_key
            .as_ref()
            .is_some_and(|key| !key.trim().is_empty())
    }

    pub fn email_alert_enabled(&self) -> bool {
        self.resend_key
            .as_ref()
            .is_some_and(|key| !key.trim().is_empty())
            && self
                .resend_email
                .as_ref()
                .is_some_and(|email| !email.trim().is_empty())
            && !self.resend_from.trim().is_empty()
    }
}
