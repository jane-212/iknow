use std::str::FromStr;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use cron::Schedule;
use tokio::time::{self, Duration};

const POLL_INTERVAL: u64 = 1;
const POLL_MAX_SIZE: usize = 5;

#[async_trait]
pub trait Task: Send + Sync {
    async fn run(&mut self) -> Result<()>;
}

struct Cron<'a> {
    task: Box<dyn Task + 'a>,
    schedule: Schedule,
    upcomings: Vec<DateTime<Local>>,
}

impl<'a> Cron<'a> {
    fn new(task: Box<dyn Task + 'a>, schedule: Schedule) -> Cron<'a> {
        let upcomings = schedule.upcoming(Local).take(POLL_MAX_SIZE).collect();
        Self {
            task,
            schedule,
            upcomings,
        }
    }
}

#[derive(Default)]
pub struct Manager<'a> {
    crons: Vec<Cron<'a>>,
}

impl<'a> Manager<'a> {
    pub fn new() -> Manager<'a> {
        Self::default()
    }

    pub fn add(mut self, cron: impl AsRef<str>, task: Box<dyn Task + 'a>) -> Result<Manager<'a>> {
        let cron = cron.as_ref();
        let sched = Schedule::from_str(cron)
            .with_context(|| format!("parse schedule from str `{}` failed", cron))?;
        self.crons.push(Cron::new(task, sched));

        Ok(self)
    }

    pub async fn start(mut self) {
        loop {
            let now = Local::now();
            for cron in self.crons.iter_mut() {
                let Some(upcoming) = cron.upcomings.first() else {
                    continue;
                };
                if upcoming < &now {
                    cron.upcomings.remove(0);
                    if let Some(upcoming) = cron.schedule.upcoming(Local).next() {
                        cron.upcomings.push(upcoming);
                    }
                    if let Err(e) = cron.task.run().await {
                        error!("{:?}", e);
                    }
                }
            }

            time::sleep(Duration::from_secs(POLL_INTERVAL)).await;
        }
    }
}
