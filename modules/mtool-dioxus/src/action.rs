use mapp::prelude::*;

pub struct Action {
    pub name: String,
    pub description: String,
   
    pub run: 
}

// #[async_trait(?Send)]
// pub trait Action {
//     async fn run(&self) -> Result<(), anyhow::Error>;
// }

// #[async_trait(?Send)]
// impl<Func, Output> Action for Func
// where
//     Func: Fn() -> Output,
//     Output: Future<Output = Result<(), anyhow::Error>>,
// {
//     async fn run(&self) -> Result<(), anyhow::Error> {
//         (self)().await
//     }
// }

// #[async_trait(?Send)]
// impl Action for Callback<(), Result<(), anyhow::Error>> {
//     async fn run(&self) -> Result<(), anyhow::Error> {
//         self.call(())
//     }
// }

struct ActionManager {
    actions: HashMap<String>,
}
