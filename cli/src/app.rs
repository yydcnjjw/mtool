use mytool_core::app::{App, Result};

use super::opts::Opts;

pub async fn run() -> Result<()> {
    let app = App::<Opts>::new()?;
    app.opts.subcmd.exec(&app).await
}