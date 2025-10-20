use emacs::{defun, Env};
use std::thread;

use crate::{context::EmacsContext, module::EmacsModule, user_idle};

emacs::plugin_is_GPL_compatible!();

#[emacs::module(name = "mtool")]
fn init(_: &Env) -> Result<(), emacs::Error> {
    Ok(())
}

#[defun(user_ptr)]
fn start(env: &Env) -> Result<EmacsContext, emacs::Error> {
    let mut builder = mapp::AppBuilder::new()?;

    let (emacs, rx) = EmacsModule::new(env)?;

    thread::spawn(move || {
        builder
            .add_module(emacs)
            .add_module(mtool_core::module())
            .add_module(mtool_storage::module())
            .add_module(mtool_p2p::module())
            .add_module(mtool_system::module())
            .add_module(user_idle::EmacsModule);

        builder.build().run();
    });

    rx.blocking_recv()?(env)
}
