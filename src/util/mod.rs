use serde::{Deserialize, Serialize};
use std::sync::Once;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Layer, Registry};

pub static INIT_LOGGER: Once = Once::new();

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    data_dir: String,
}

pub fn init_console_logger() {
    INIT_LOGGER.call_once(|| {
        let stdout_log = tracing_subscriber::fmt::layer().pretty();
        let subscriber = Registry::default().with(stdout_log.with_filter(LevelFilter::TRACE));

        tracing::subscriber::set_global_default(subscriber).expect("Console log init failed!");
    });
}
