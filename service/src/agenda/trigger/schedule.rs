use std::str::FromStr;

use cron::Schedule;
use log::{error, info};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Schedule parse: {0}")]
    ScheduleParse(#[from] cron::error::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

struct ScheduleTrigger {
    schedule: Schedule,
}

impl ScheduleTrigger {
    fn new(schedule: Schedule) -> Self {
        Self { schedule }
    }

    fn new(schedule: String) -> Result<Self> {
        Ok(Self {
            schedule: Schedule::from_str(&schedule)?,
        })
    }

    async fn run(&self) -> Result<()> {
        for dt in self.schedule.upcoming(chrono::Utc) {
            let duration = dt.signed_duration_since(chrono::Utc::now());
            time::sleep(time::Duration::from_millis(
                duration.num_milliseconds() as u64
            ))
            .await;

            // TODO: trigger
            // match self.task.run().await {
            //     Ok(_) => info!("Executing task {} successfully", self.task),
            //     Err(e) => {
            //         // TODO: notification
            //         error!("Executing task {} failed: {}", self.task, e);
            //     }
            // }
        }
        Ok(())
    }
}
