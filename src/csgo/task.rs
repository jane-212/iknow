use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{Days, Local, NaiveDate};
use colored::Colorize;

use crate::csgo::api::{CsgoApi, Match};
use crate::utils::{Mail, Task};
use crate::TEMPLATES;

#[derive(Clone)]
pub struct Csgo {
    api: CsgoApi,
    mail: Mail,
}

impl Csgo {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        from: impl Into<String>,
        reply_to: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<Csgo> {
        let api = CsgoApi::new().context("init csgo api failed")?;
        let mail = Mail::new(username, password, from, reply_to, to).context("init mail failed")?;

        Ok(Self { api, mail })
    }
}

#[async_trait]
impl Task for Csgo {
    async fn run(&mut self) -> Result<()> {
        info!("run task `{}`", "csgo".green().bold());

        let today = Local::now().date_naive();
        let days = vec![
            Some(today),
            today.checked_add_days(Days::new(1)),
            today.checked_add_days(Days::new(2)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<NaiveDate>>();

        let mut matches = Vec::new();
        for day in days {
            matches.push(
                self.api
                    .get_matches_by_date(&day)
                    .await
                    .with_context(|| format!("get matches of `{}` failed", &day))?,
            );
        }
        let matches = matches.into_iter().flatten().collect::<Vec<Match>>();
        info!("get all matches {}", "successfully".green().bold());

        let mut context = tera::Context::new();
        context.insert("matches", &matches);
        let content = TEMPLATES
            .render("csgo.html", &context)
            .context("render template `csgo.html` failed")?;
        self.mail
            .send("csgo matches near 3 days", content)
            .context("send csgo mail failed")?;
        info!("send mail {}", "successfully".green().bold());

        Ok(())
    }
}
