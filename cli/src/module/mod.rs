mod test;
mod translate;
mod sysev;

use crate::app::App;

pub async fn module_load(app: &mut App) -> anyhow::Result<()> {
    test::module_load(app).await?;
    translate::module_load(app).await?;
    Ok(())
}
