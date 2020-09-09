use std::{
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use chrono::{DateTime, TimeZone};
use cron::Schedule;
use futures::Future;
use log::error;
use thiserror::Error;

use serde::{Deserialize, Serialize};
use tokio::time::{self, sleep, Sleep};
use tokio_stream::Stream;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Schedule parse: {0}")]
    ScheduleParse(#[from] cron::error::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ScheduleTrigger {
    #[serde(with = "schedule")]
    schedule: Schedule,

    #[serde(skip)]
    sleep: Option<Pin<Box<Sleep>>>,
}

impl ScheduleTrigger {
    pub fn new(schedule: Schedule) -> Self {
        Self {
            schedule,
            sleep: None,
        }
    }

    pub fn from(schedule: String) -> Result<Self> {
        Ok(Self::new(Schedule::from_str(&schedule)?))
    }

    fn upcoming(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.schedule.upcoming(chrono::Utc).next()
    }
}

trait DateTimeExt<Tz>
where
    Tz: TimeZone,
{
    fn since_now_ms(self) -> u64;
}

impl<Tz> DateTimeExt<Tz> for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn since_now_ms(self) -> u64 {
        let tz = self.timezone();
        self.signed_duration_since(chrono::Utc::now().with_timezone(&tz))
            .num_milliseconds() as u64
    }
}

impl Stream for ScheduleTrigger {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if self.sleep.is_none() {
            match self.upcoming() {
                Some(dt) => {
                    self.sleep = Some(Box::pin(sleep(time::Duration::from_millis(
                        dt.since_now_ms(),
                    ))));
                }
                None => {
                    return Poll::Ready(None);
                }
            }
        }

        let s = self.sleep.as_mut().unwrap();

        match s.as_mut().poll(cx) {
            Poll::Ready(_) => match self.upcoming() {
                Some(dt) => {
                    self.sleep = Some(Box::pin(sleep(time::Duration::from_millis(
                        dt.since_now_ms(),
                    ))));
                    Poll::Ready(Some(()))
                }
                None => {
                    return Poll::Ready(None);
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

mod schedule {
    use std::str::FromStr;

    use cron::Schedule;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(sche: &Schedule, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&sche.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Schedule, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Schedule::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::trigger::ScheduleTrigger;


    #[test]
    fn test_schedule_serialize() {
        let sche = ScheduleTrigger::from("* * * * * * *".to_string()).unwrap();
        assert!(toml::to_string_pretty(&sche).is_ok());
    }

    #[test]
    fn test_schedule_deserialize() {
        assert!(toml::from_str::<ScheduleTrigger>("schedule = \"* * * * * * *\"").is_ok());
    }
}
