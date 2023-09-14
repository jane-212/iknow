use std::env;
use std::io::Write;

use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use dotenv::dotenv;
use env_logger::Builder;
use log::Level;
use shadow_rs::shadow;
use tokio::signal::unix::{signal, SignalKind};

use iknow::csgo::Csgo;
use iknow::utils::Manager;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    dotenv().ok();
    Builder::from_default_env()
        .format(|buf, record| {
            let level = record.level().to_string();

            let level = match record.level() {
                Level::Error => level.red(),
                Level::Warn => level.yellow(),
                Level::Info => level.green(),
                Level::Debug => level.purple(),
                Level::Trace => level.blue(),
            }
            .bold();

            writeln!(
                buf,
                "[{} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                level,
                record.args(),
            )
        })
        .init();

    if let Err(e) = entry().await {
        error!("{:?}", e);
    }
}

async fn entry() -> Result<()> {
    show_banner();

    let mail_username = env::var("MAIL_USERNAME").context("MAIL_USERNAME missing")?;
    let mail_password = env::var("MAIL_PASSWORD").context("MAIL_PASSWORD missing")?;
    let mail_from = env::var("MAIL_FROM").context("MAIL_FROM missing")?;
    let mail_reply_to = env::var("MAIL_REPLY_TO").context("MAIL_REPLY_TO missing")?;
    let mail_to = env::var("MAIL_TO").context("MAIL_TO missing")?;

    run(
        mail_username,
        mail_password,
        mail_from,
        mail_reply_to,
        mail_to,
    )
    .await
    .context("run app failed")?;

    info!("app {}", "start".green().bold());

    let mut sigint = signal(SignalKind::interrupt()).context("create signal interrupt failed")?;
    let mut sigterm = signal(SignalKind::terminate()).context("create signal terminate failed")?;
    tokio::select! {
        _ = sigint.recv() => {
            info!("receive signal {}", "interrupt".yellow().bold());
        }
        _ = sigterm.recv() => {
            info!("receive signal {}", "terminate".yellow().bold());
        }
    }

    info!("app {}", "quit".red().bold());

    Ok(())
}

shadow!(build);

fn show_banner() {
    macro_rules! new_line {
        ($banner: expr) => {
            $banner.push('\n')
        };
        ($banner: expr, $line: expr) => {
            $banner.push('\n');
            $banner.push_str($line)
        };
    }

    let logo = include_str!("../iknow.banner");
    let mut banner = String::new();

    new_line!(banner);
    new_line!(banner, &format!("{}", logo.yellow().bold()));
    new_line!(banner);
    new_line!(
        banner,
        &format!(
            "{} {}",
            "name:".blue().bold(),
            build::PROJECT_NAME.yellow().bold()
        )
    );
    new_line!(
        banner,
        &format!(
            "{} {}",
            "version:".blue().bold(),
            build::PKG_VERSION.yellow().bold()
        )
    );
    new_line!(
        banner,
        &format!(
            "{} {}",
            "description:".blue().bold(),
            build::PKG_DESCRIPTION.yellow().bold()
        )
    );
    new_line!(
        banner,
        &format!(
            "{} {}",
            "production:".blue().bold(),
            build::BUILD_RUST_CHANNEL.yellow().bold()
        )
    );
    new_line!(
        banner,
        &format!(
            "{} {}",
            "target_os:".blue().bold(),
            build::BUILD_OS.yellow().bold()
        )
    );
    new_line!(
        banner,
        &format!(
            "{} {}",
            "build_env:".blue().bold(),
            build::BUILD_TARGET.yellow().bold()
        )
    );
    new_line!(banner);
    info!("{}", banner);
}

async fn run(
    username: impl Into<String>,
    password: impl Into<String>,
    from: impl Into<String>,
    reply_to: impl Into<String>,
    to: impl Into<String>,
) -> Result<()> {
    let csgo = Csgo::new(username, password, from, reply_to, to).context("init csgo failed")?;
    #[cfg(debug_assertions)]
    let manager = Manager::new()
        .add("*/10 * * * * *", Box::new(csgo))
        .context("add cron job failed")?;
    #[cfg(not(debug_assertions))]
    let manager = Manager::new()
        .add("0 0 1 * * *", Box::new(csgo))
        .context("add cron job failed")?;
    info!("add cron job `{}`", "csgo".green().bold());
    tokio::spawn(async move {
        manager.start().await;
    });

    Ok(())
}
