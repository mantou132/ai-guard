use std::sync::Arc;

use crate::{config::Config, store::Store};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub store: Store,
    pub client: reqwest::Client,
}
