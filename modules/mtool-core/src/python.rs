use std::{any::type_name, sync};

use mapp::anyhow::{self, Context};
use pyo3::prelude::*;

static INIT: sync::Once = sync::Once::new();

pub fn py_run<F, R>(f: F) -> Result<R, anyhow::Error>
where
    F: for<'py> FnOnce(Python<'py>) -> PyResult<R> + 'static,
    R: 'static,
{
    Python::with_gil(move |py| {
        INIT.call_once(|| {
            move || -> PyResult<()> {
                let signal = py.import("signal")?;
                signal
                    .getattr("signal")?
                    .call1((signal.getattr("SIGINT")?, signal.getattr("SIG_DFL")?))?;
                Ok(())
            }()
            .expect("python init failed");
        });
        f(py).context(format!("{}", type_name::<F>()))
    })
}
