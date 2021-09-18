use super::{operate::ShellOperate, trigger::ScheduleTrigger};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    name: String,
    description: String,
    triggers: Vec<Trigger>,
    operates: Vec<Operate>,
}

impl Task {
    fn new(
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
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Operate {
    Shell(ShellOperate),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Trigger {
    Schedule(ScheduleTrigger),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task() {
        let task = Task::new(
            String::from("test"),
            String::from("test"),
            vec![
                Trigger::Schedule(ScheduleTrigger::from(String::from("* * * * * * *")).unwrap()),
                Trigger::Schedule(ScheduleTrigger::from(String::from("*/5 * * * * * *")).unwrap()),
            ],
            vec![Operate::Shell(ShellOperate::new(String::from("echo test")))],
        );

        println!("{}", toml::to_string_pretty(&task).unwrap());
    }
}
