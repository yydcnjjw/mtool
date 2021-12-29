use crate::Event;

mod key;
mod record;

pub fn run_loop<F>(cb: F) -> anyhow::Result<()>
where
    F: 'static + Fn(Event) + Send + Sync,
{
    Ok(record::Record::run_loop(cb)?)
}

pub fn quit() -> anyhow::Result<()> {
    Ok(record::Record::quit()?)
}
