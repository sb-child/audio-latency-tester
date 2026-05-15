use audio_latency_tester::{
    app,
    config::AppConfig,
    error::{AppError, ConfigParseSnafu, ConfigReadSnafu},
};

use clap::Parser;
use snafu::ResultExt;
use std::path::PathBuf;

use tracing::{Level, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
}

fn load_config(path: &PathBuf) -> Result<AppConfig, AppError> {
    let contents = std::fs::read_to_string(path).context(ConfigReadSnafu { path })?;
    toml::from_str(&contents).context(ConfigParseSnafu)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(true)
        .init();

    let cli = Cli::parse();
    info!("加载配置文件: {:?}", cli.config);
    let app_config = load_config(&cli.config)?;

    app::app(app_config)?;

    Ok(())
}
