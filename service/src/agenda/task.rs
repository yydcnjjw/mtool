use super::operate;
use super::{operate::ShellOperate, trigger::ScheduleTrigger};

struct Task {
    name: String,
    description: String,
    trigger: ScheduleTrigger,
    operate: ShellOperate,
}
