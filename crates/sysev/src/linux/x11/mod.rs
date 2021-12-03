use crate::Event;

mod key;
mod record;

pub fn run_loop<F>(cb: F) -> anyhow::Result<()>
where
    F: Fn(Event),
{
    Ok(record::Record::run_loop(cb)?)
}
