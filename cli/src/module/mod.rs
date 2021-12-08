mod test;
mod translate;
mod gterminal;

use crate::app::App;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    test::module_load(app).await?;
    translate::module_load(app).await?;
    gterminal::module_load(app).await?;
    Ok(())
}
