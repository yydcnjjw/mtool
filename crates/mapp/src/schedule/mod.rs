mod schedule;
mod task;

pub use schedule::*;
pub use task::*;

#[cfg(test)]
mod tests {
    use crate::{define_label, App, Label};

    use super::*;

    define_label!(
        enum StartupStage {
            PreStartup,
            Startup,
            PostStartup,
        }
    );

    #[tokio::test]
    async fn test_schedule() {
        let schedule = Schedule::new();
        let app = App::new();
        schedule
            .add_stage(StartupStage::PreStartup)
            .await
            .add_stage(StartupStage::Startup)
            .await
            .add_stage(StartupStage::PostStartup)
            .await
            .add_task(
                StartupStage::PreStartup,
                async move || -> Result<(), anyhow::Error> {
                    println!("PreStartup");
                    Ok(())
                },
            )
            .await
            .add_task(
                StartupStage::Startup,
                async move || -> Result<(), anyhow::Error> {
                    println!("Startup");
                    Ok(())
                },
            )
            .await
            .add_task(
                StartupStage::PostStartup,
                async move || -> Result<(), anyhow::Error> {
                    println!("PostStartup");
                    Ok(())
                },
            )
            .await;

        schedule.run(&app).await.unwrap();
    }
}
