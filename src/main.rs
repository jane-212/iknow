use std::env;
use std::io::Write;

use anyhow::Result;
use chrono::Local;
use dotenv::dotenv;
use env_logger::{fmt::Color, Builder};
use tokio::signal::unix::{signal, SignalKind};

use iknow::csgo::Csgo;
use iknow::utils::Manager;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
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

    let mail_username = env::var("MAIL_USERNAME")?;
    let mail_password = env::var("MAIL_PASSWORD")?;
    let mail_from = env::var("MAIL_FROM")?;
    let mail_reply_to = env::var("MAIL_REPLY_TO")?;
    let mail_to = env::var("MAIL_TO")?;

    run(
        mail_username,
        mail_password,
        mail_from,
        mail_reply_to,
        mail_to,
    )
    .await?;

    info!("app start");

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
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

async fn run(
    username: impl Into<String>,
    password: impl Into<String>,
    from: impl Into<String>,
    reply_to: impl Into<String>,
    to: impl Into<String>,
) -> Result<()> {
    let csgo = Csgo::new(username, password, from, reply_to, to)?;
    #[cfg(debug_assertions)]
    let manager = Manager::new().add("*/10 * * * * *", Box::new(csgo))?;
    #[cfg(not(debug_assertions))]
    let manager = Manager::new().add("0 1 * * * *", Box::new(csgo))?;
    tokio::spawn(async move {
        manager.start().await;
    });

    Ok(())
}
