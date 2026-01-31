// use std::collections::{HashMap, LinkedList};

// use async_recursion::async_recursion;
// use indextree::{Arena, NodeId};

// use crate::{
//     context::{Context, ContextError},
//     coroutine::Callable,
// };

// use super::task::ScheduleTask;

// type Condition = Box<dyn Callable<Output = bool, Error = ContextError>>;

// struct Node {
//     task: Option<ScheduleTask>,
//     condition: Option<Condition>,
// }

// impl Node {
//     fn empty() -> Self {
//         Self {
//             task: None,
//             condition: None,
//         }
//     }
// }

use super::{config::IntoScheduleConfigs, set::ScheduleLabel, task::ScheduleTask};

#[derive(Debug, Default)]
pub struct Scheduler {}

impl Scheduler {
    pub fn add_tasks<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        tasks: impl IntoScheduleConfigs<ScheduleTask, M>,
    ) -> &mut Self {
        tasks.into_configs();
        self
    }
}

// impl Scheduler {
//     pub fn new() -> Self {
//         let mut tree = Arena::new();
//         let mut nodes = HashMap::new();

//         let root = tree.new_node(Node::empty());
//         {
//             nodes.insert(ScheduleGraph::Root.into(), root);
//         }

//         Scheduler { tree, nodes, root }
//     }

//     pub fn insert_before(&mut self, label: Label, node: Node) {
//         // let node = self.tree.new_node(data);
//     }

//     pub fn insert_after(&mut self, label: Label, node: Node) {}

//     pub fn add(&mut self, label: Label, node: Node) {}

//     pub async fn run(mut self, ctx: &Context) -> Result<(), ContextError> {
//         self.run_node(self.root, ctx).await
//     }

//     #[async_recursion(?Send)]
//     pub async fn run_node(&mut self, node_id: NodeId, ctx: &Context) -> Result<(), ContextError> {
//         let node = &mut self.tree[node_id];
//         let node = node.get_mut();

//         if let Some(cond) = node.condition.take()
//             && !cond.call(ctx).await?
//         {
//             return Ok(());
//         }

//         if let Some(task) = node.task.take() {
//             task.run(ctx).await?;
//         }

//         for id in self.root.children(&self.tree) {
//             let node = &self.tree[id];
//         }

//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[derive(ScheduleLabel, Hash, Debug, Clone, PartialEq, Eq)]
    struct TestSchedule;

    #[derive(TaskSet, Debug, Hash, Clone, PartialEq, Eq)]
    enum TestSet {
        Startup,
        First,
        After,
    }

    #[test]
    fn tasks() {
        let mut schedules = Scheduler::default();

        async fn test() -> Result<(), ContextError> {
            Ok(())
        }

        schedules.add_tasks(
            TestSchedule,
            (async move || Ok::<_, ContextError>(())).in_set(TestSet::First),
        );

        schedules.add_tasks(
            TestSchedule,
            test.in_set(TestSet::First)
                .before(TestSet::After)
                .after(TestSet::Startup),
        );
    }
}
