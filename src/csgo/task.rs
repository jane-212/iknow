use anyhow::Result;
use async_trait::async_trait;
use chrono::{Days, Local, NaiveDate};
use tera::Context;

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
        let api = CsgoApi::new()?;
        let mail = Mail::new(username, password, from, reply_to, to)?;

        Ok(Self { api, mail })
    }
}

#[async_trait]
impl Task for Csgo {
    async fn run(&mut self) -> Result<()> {
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
            matches.push(self.api.get_matches_by_date(&day).await?);
        }

        let matches = matches.into_iter().flatten().collect::<Vec<Match>>();

        let mut context = Context::new();
        context.insert("matches", &matches);
        let content = TEMPLATES.render("csgo.html", &context)?;
        self.mail.send("csgo matches near 3 days", content)?;

        Ok(())
    }
}
