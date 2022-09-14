mod schedule;

pub use schedule::ScheduleTrigger;

enum Trigger {
    Schedule(ScheduleTrigger),
}
