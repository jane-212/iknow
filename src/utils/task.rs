use std::str::FromStr;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use colored::Colorize;
use cron::Schedule;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};
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
    task_name: String,
    schedule_description: String,
}

impl<'a> Cron<'a> {
    fn new(
        task: Box<dyn Task + 'a>,
        schedule: Schedule,
        task_name: impl Into<String>,
        schedule_description: impl Into<String>,
    ) -> Cron<'a> {
        let upcomings = schedule.upcoming(Local).take(POLL_MAX_SIZE).collect();
        let task_name = task_name.into();
        let schedule_description = schedule_description.into();
        Self {
            task,
            schedule,
            upcomings,
            task_name,
            schedule_description,
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

    pub fn add(
        mut self,
        cron: impl AsRef<str>,
        name: impl Into<String>,
        task: Box<dyn Task + 'a>,
    ) -> Result<Manager<'a>> {
        let cron = cron.as_ref();
        let sched = Schedule::from_str(cron)
            .with_context(|| format!("parse schedule from str `{}` failed", cron))?;
        self.crons.push(Cron::new(task, sched, name.into(), cron));

        Ok(self)
    }

    fn show_info(&self) {
        let mut table = Table::new();
        table.style = TableStyle::blank();
        let tag_align = Alignment::Right;
        let content_align = Alignment::Left;

        table.add_row(Row::new(vec![
            TableCell::new_with_alignment("task".blue().bold(), 1, tag_align),
            TableCell::new_with_alignment("schedule".yellow().bold(), 1, content_align),
        ]));
        for cron in self.crons.iter() {
            table.add_row(Row::new(vec![
                TableCell::new_with_alignment(cron.task_name.blue().bold(), 1, tag_align),
                TableCell::new_with_alignment(
                    cron.schedule_description.yellow().bold(),
                    1,
                    content_align,
                ),
            ]));
        }

        info!("\n\n{}", table.render());
    }

    pub async fn start(mut self) {
        self.show_info();

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
