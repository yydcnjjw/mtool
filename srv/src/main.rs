use log::debug;

use mytool_service::agenda::operate::{ShellOperate, AsyncOperate};
use tokio::time;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let op = ShellOperate::new(String::from("echo test"));
    op.run().await.unwrap();

    // let task = TimedTask::new(
    //     "* * * * * * *",
    //     Box::new(ShellTask::new(String::from("echo test"))),
    // )
    // .unwrap();

    // // let task2 = Arc::new(TimedTask::new(
    // //     cron::Schedule::from_str("* * * * * * *").unwrap(),
    // //     sh_script_task(String::from("echo demo")),
    // // ));

    // tokio::spawn(async move { task.run().await });
    // // tokio::spawn(async move { task2.run().await });

    // loop {
    //     debug!("main");
    //     time::sleep(time::Duration::from_millis(1000)).await;
    // }
}
