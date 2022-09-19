use crate::BoxedEventCallback;

use super::record;

pub fn run_loop(cb: BoxedEventCallback) -> Result<(), anyhow::Error> {
    Ok(record::run_loop(cb)?)
}

pub fn quit() -> Result<(), anyhow::Error> {
    Ok(record::quit()?)
}
