use crate::EventCallback;

mod key;
mod record;

pub fn run_loop(cb: EventCallback) -> anyhow::Result<()> {
    Ok(record::Record::run_loop(cb)?)
}
