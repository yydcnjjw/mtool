pub mod event_loop;
mod wayland;

// use std::env;

// mod wayland;
// mod x11;

// pub fn run_loop<F>(#[allow(unused)] cb: F) -> Result<(), anyhow::Error>
// where
//     F: Fn(Event) -> Result<(), anyhow::Error> + Send + Sync + 'static,
// {
//     // env::var("XDG_SESSION_TYPE")
//     wayland::run_loop(cb)?;
//     x11::event::run_loop(cb)?;
//     Ok(())
// }

// pub fn quit() -> Result<(), anyhow::Error> {
//     Ok(())
// }
