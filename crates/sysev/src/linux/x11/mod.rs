use crate::EventSender;

mod key;
mod record;

pub fn run_loop(sender: EventSender) -> anyhow::Result<()> {
    Ok(record::Record::run_loop(sender)?)
}
