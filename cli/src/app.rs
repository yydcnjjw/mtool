use mytool_core::app::{App, Result};

use super::opts::Opts;

pub async fn run() -> Result<()> {
    App::<Opts>::new()?.run().await
}
