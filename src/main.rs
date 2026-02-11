#![allow(dead_code)]

mod action;
mod api;
mod app;
mod config;
mod event;
mod models;
mod tui;
mod ui;
mod utils;

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "skyscraper", version, about = "A TUI client for Bluesky")]
struct Cli {
    /// Bluesky handle (e.g. alice.bsky.social)
    #[arg(short = 'u', long)]
    handle: Option<String>,

    /// Use app password authentication instead of OAuth
    #[arg(long)]
    app_password: bool,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "error")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cli.log_level)),
        )
        .with_writer(std::io::stderr)
        .init();

    let client = Arc::new(api::client::BlueskyClient::new().await?);

    let mut terminal = tui::init()?;

    // Install a panic hook that restores the terminal before printing the panic
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = tui::restore();
        default_panic(info);
    }));

    let result = app::App::new(cli.handle, cli.app_password, client).run(&mut terminal).await;
    tui::restore()?;

    result
}
