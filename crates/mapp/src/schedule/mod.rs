mod schedule;
mod once_task;
mod cond_load;

pub use schedule::*;
pub use once_task::*;
pub use cond_load::*;

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
            .add_task(StartupStage::PreStartup, || async move {
                println!("PreStartup");
                Ok::<(), anyhow::Error>(())
            })
            .await
            .add_task(StartupStage::Startup, || async move {
                println!("Startup");
                Ok::<(), anyhow::Error>(())
            })
            .await
            .add_task(StartupStage::PostStartup, || async move {
                println!("PostStartup");
                Ok::<(), anyhow::Error>(())
            })
            .await;

        schedule.run(&app).await.unwrap();
    }
}
