use crate::event_bus::Sender;

mod key;
mod record;

pub fn run_loop(sender: Sender) -> anyhow::Result<()> {
    Ok(record::Record::run_loop(sender)?)
}
