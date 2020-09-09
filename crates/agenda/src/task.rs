use std::{
    fmt::Display,
    pin::Pin,
    task::{self, Poll},
};

use crate::operate;

use super::{
    operate::{AsyncOperate, ShellOperate},
    trigger::ScheduleTrigger,
};

use anyhow::Context;
use async_trait::async_trait;
use futures::future::join_all;
use log::error;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_stream::{Stream, StreamExt, StreamMap};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    name: String,
    description: String,
    triggers: Vec<Trigger>,
    operates: Vec<Operate>,
}

impl Task {
    pub fn new(
        name: String,
        description: String,
        triggers: Vec<Trigger>,
        operates: Vec<Operate>,
    ) -> Self {
        Self {
            name,
            description,
            triggers,
            operates,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut map = StreamMap::new();
        self.triggers.iter_mut().enumerate().for_each(|(i, item)| {
            map.insert(i, Pin::new(item));
        });

        loop {
            tokio::select! {
                Some(_) = map.next() => {
                    for result in join_all(self.operates.iter().map(|op| op.run())).await {
                        if let Err(e) = result.with_context(|| format!("Task {}", self.name)) {
                            error!("{:?}", e);
                        }
                    }
                }
                else => break,
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Operate {
    Shell(ShellOperate),
}

#[async_trait]
impl AsyncOperate for Operate {
    async fn run(&self) -> operate::AsyncResult<()> {
        match self {
            Operate::Shell(v) => v.run().await,
        }
    }
}

impl Display for Operate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Trigger {
    Schedule(ScheduleTrigger),
}

impl Stream for Trigger {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut *self {
            Trigger::Schedule(v) => Pin::new(v).poll_next(cx),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{operate::ShellOperate, trigger::ScheduleTrigger, Operate, Task, Trigger};

    #[test]
    fn test_task() {
        let tasks = vec![
            Task::new(
                String::from("test"),
                String::from("test"),
                vec![Trigger::Schedule(
                    ScheduleTrigger::from(String::from("* * * * * * *")).unwrap(),
                )],
                vec![Operate::Shell(ShellOperate::new(String::from("echo test")))],
            ),
            Task::new(
                String::from("test2"),
                String::from("test2"),
                vec![Trigger::Schedule(
                    ScheduleTrigger::from(String::from("* * * * * * *")).unwrap(),
                )],
                vec![Operate::Shell(ShellOperate::new(String::from("echo test")))],
            ),
        ];

        let text = toml::to_string_pretty(
            &tasks
                .iter()
                .map(|item| (item.name.clone(), item))
                .collect::<HashMap<_, _>>(),
        )
        .unwrap();
        toml::from_str::<HashMap<String, Task>>(&text).unwrap();
    }
}
