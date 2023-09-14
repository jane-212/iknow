use std::env;
use std::io::Write;

use anyhow::{Context, Result};
use chrono::Local;
use dotenv::dotenv;
use env_logger::{fmt::Color, Builder};
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
            let mut style = buf.style();
            let level = record.level();

            let style = match level {
                log::Level::Error => style.set_color(Color::Red),
                log::Level::Warn => style.set_color(Color::Yellow),
                log::Level::Info => style.set_color(Color::Green),
                log::Level::Debug => style.set_color(Color::Magenta),
                log::Level::Trace => style.set_color(Color::Blue),
            };
            style.set_bold(true);

            writeln!(
                buf,
                "[{} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                style.value(level),
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

    info!("load environment variables");

    run(
        mail_username,
        mail_password,
        mail_from,
        mail_reply_to,
        mail_to,
    )
    .await
    .context("run app failed")?;

    info!("app start");

    let mut sigint = signal(SignalKind::interrupt()).context("create signal interrupt failed")?;
    let mut sigterm = signal(SignalKind::terminate()).context("create signal terminate failed")?;
    tokio::select! {
        _ = sigint.recv() => {
            info!("receive signal interrupt");
        }
        _ = sigterm.recv() => {
            info!("receive signal terminate");
        }
    }

    info!("app quit");

    Ok(())
}

shadow!(build);

fn show_banner() {
    let banner = include_str!("../iknow.banner");
    info!(
        "\n\n{}\n\nname: {}\nversion: {}\ndescription: {}\nproduction: {}\ntarget_os: {}\nbuild_time: {}\nbuild_env: {}\n",
        banner,
        build::PROJECT_NAME,
        build::PKG_VERSION,
        build::PKG_DESCRIPTION,
        build::BUILD_RUST_CHANNEL,
        build::BUILD_OS,
        build::BUILD_TIME,
        build::BUILD_TARGET
    );
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
    info!("add cron job csgo");
    tokio::spawn(async move {
        manager.start().await;
    });

    Ok(())
}
