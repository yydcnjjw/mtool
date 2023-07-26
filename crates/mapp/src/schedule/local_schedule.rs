use std::{cell::RefCell, collections::HashMap, mem};

use anyhow::Context;
use async_recursion::async_recursion;
use futures::Future;
use minject::{InjectOnce, LocalProvide};
use petgraph::{graph::NodeIndex, Direction, Graph};
use tracing::debug;

use crate::{
    FnCondLoad, IntoLocalOnceTaskDescriptor, Label, LocalApp, LocalCondLoad,
    LocalOnceTaskDescriptor, ScheduleGraph,
};

enum Node {
    OnceTask(OnceTaskNode),
    Stage(StageNode),
}

struct OnceTaskNode {
    task: RefCell<Option<LocalOnceTaskDescriptor>>,
    index: NodeIndex,
}

impl OnceTaskNode {
    fn new(task: LocalOnceTaskDescriptor, idx: NodeIndex) -> Self {
        Self {
            task: RefCell::new(Some(task)),
            index: idx,
        }
    }

    async fn run_once(&self, app: &LocalApp) -> Result<(), anyhow::Error> {
        let task = self.task.borrow_mut().take().context("task is not exist")?;
        task.run_once(app).await
    }
}

type BoxedLocalCondLoad = Box<dyn LocalCondLoad>;

struct StageNode {
    index: NodeIndex,
    cond_load: RefCell<Option<BoxedLocalCondLoad>>,
}

impl StageNode {
    fn new(idx: NodeIndex) -> Self {
        Self {
            index: idx,
            cond_load: RefCell::new(None),
        }
    }

    fn new_with_cond(idx: NodeIndex, cond_load: Option<BoxedLocalCondLoad>) -> Self {
        Self {
            index: idx,
            cond_load: RefCell::new(cond_load),
        }
    }
}

type SchedGraph = Graph<Label, ()>;

#[derive(Default)]
struct ScheduleInner {
    graph: SchedGraph,
    node_index: HashMap<Label, Node>,
}

impl ScheduleInner {
    fn new() -> Self {
        let graph = SchedGraph::new();
        let node_index = HashMap::new();

        let mut self_ = Self { graph, node_index };

        let root = ScheduleGraph::Root.into();
        let stage = self_.graph.add_node(root);
        self_
            .node_index
            .insert(root, Node::Stage(StageNode::new(stage)));

        self_
    }
}

impl ScheduleInner {
    fn insert_stage<L, B>(
        &mut self,
        prev_stage_label: L,
        stage_label: B,
        cond_load: Option<BoxedLocalCondLoad>,
    ) -> &mut Self
    where
        L: Into<Label>,
        B: Into<Label>,
    {
        let stage_label = stage_label.into();

        let stage = self.graph.add_node(stage_label);
        self.node_index.insert(
            stage_label,
            Node::Stage(StageNode::new_with_cond(stage, cond_load)),
        );

        let prev_stage_index = self.get_stage(prev_stage_label).unwrap().index;

        for next_stage in self
            .graph
            .neighbors_directed(prev_stage_index, Direction::Outgoing)
            .filter(|node_index| {
                matches!(self.get_node_with_index(node_index.clone()), Node::Stage(_))
            })
            .collect::<Vec<_>>()
        {
            let edge = self.graph.find_edge(prev_stage_index, next_stage).unwrap();
            self.graph.remove_edge(edge);
            self.graph.add_edge(stage, next_stage, ());
        }

        self.graph.update_edge(prev_stage_index, stage, ());

        self
    }

    fn insert_stage_vec<L, B>(
        &mut self,
        prev_stage: L,
        stages: Vec<(B, Option<BoxedLocalCondLoad>)>,
    ) -> &mut Self
    where
        L: Into<Label>,
        B: Into<Label>,
    {
        let mut prev = prev_stage.into();
        for (stage, cond_load) in stages {
            let stage = stage.into();
            self.insert_stage(prev, stage.clone(), cond_load);
            prev = stage;
        }

        self
    }

    fn add_once_task<L>(&mut self, label: L, task: LocalOnceTaskDescriptor) -> &mut Self
    where
        L: Into<Label>,
    {
        let label = label.into();
        let index = self
            .get_node_index(label.clone())
            .expect(&format!("{} is not exist", &label));

        let task_label = task.label;

        let task_node = self.graph.add_node(task_label);

        self.graph.add_edge(index, task_node, ());

        self.node_index.insert(
            task_label,
            Node::OnceTask(OnceTaskNode::new(task, task_node)),
        );

        self
    }

    fn get_stage<L>(&self, stage_label: L) -> Option<&StageNode>
    where
        L: Into<Label>,
    {
        self.node_index
            .get(&stage_label.into())
            .and_then(|v| match v {
                Node::OnceTask(_) => None,
                Node::Stage(n) => Some(n),
            })
    }

    fn get_node_with_index(&self, index: NodeIndex) -> &Node {
        self.node_index
            .get(self.graph.node_weight(index).unwrap())
            .unwrap()
    }

    fn get_node_index<L>(&self, label: L) -> Option<NodeIndex>
    where
        L: Into<Label>,
    {
        self.node_index.get(&label.into()).and_then(|v| match v {
            Node::OnceTask(t) => Some(t.index),
            Node::Stage(s) => Some(s.index),
        })
    }

    pub async fn run(self, app: &LocalApp) -> Result<(), anyhow::Error> {
        let root_stage = self.get_stage(ScheduleGraph::Root).unwrap();
        let neighbors = self
            .graph
            .neighbors_directed(root_stage.index, Direction::Outgoing);

        self.run_node_parallel(app, neighbors).await
    }

    async fn run_node_parallel<Iter>(&self, app: &LocalApp, iter: Iter) -> Result<(), anyhow::Error>
    where
        Iter: IntoIterator<Item = NodeIndex>,
    {
        let tasks = iter
            .into_iter()
            .map(|node| self.run_node(app, node))
            .collect::<Vec<_>>();

        futures::future::try_join_all(tasks).await.map(|_| ())
    }

    #[async_recursion(?Send)]
    async fn run_node(&self, app: &LocalApp, index: NodeIndex) -> Result<(), anyhow::Error> {
        let label = self.graph.node_weight(index).unwrap();
        let node = self.node_index.get(&label).unwrap();
        match node {
            Node::OnceTask(task) => {
                task.run_once(app).await?;

                self.run_node_parallel(
                    app,
                    self.graph.neighbors_directed(index, Direction::Outgoing),
                )
                .await?;
            }
            Node::Stage(stage) => {
                let mut stage = stage.cond_load.borrow_mut();
                let need_run_task = match stage.take() {
                    Some(cond_load) => cond_load.local_load_with_cond(app).await?,
                    _ => true,
                };

                if need_run_task {
                    debug!("run stage: {}", label);
                    self.run_node_parallel(
                        app,
                        self.graph
                            .neighbors_directed(index, Direction::Outgoing)
                            .filter(|index| {
                                matches!(self.get_node_with_index(*index), Node::OnceTask(_))
                            }),
                    )
                    .await?;
                }

                self.run_node_parallel(
                    app,
                    self.graph
                        .neighbors_directed(index, Direction::Outgoing)
                        .filter(|index| matches!(self.get_node_with_index(*index), Node::Stage(_))),
                )
                .await?;
            }
        };
        Ok(())
    }
}

pub struct LocalSchedule {
    inner: RefCell<ScheduleInner>,
}

impl LocalSchedule {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ScheduleInner::new()),
        }
    }

    pub fn insert_stage<L, B>(&self, prev_stage_label: L, stage_label: B) -> &Self
    where
        L: Into<Label>,
        B: Into<Label>,
    {
        self.inner
            .borrow_mut()
            .insert_stage(prev_stage_label, stage_label, None);
        self
    }

    pub fn insert_stage_with_cond<L, B, Func, Args, Output>(
        &self,
        prev_stage_label: L,
        stage_label: B,
        cond_load: Func,
    ) -> &Self
    where
        L: Into<Label>,
        B: Into<Label>,
        Func: InjectOnce<Args, Output = Output> + 'static,
        Args: LocalProvide<LocalApp> + Send + Sync + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>> + Send,
    {
        self.inner.borrow_mut().insert_stage(
            prev_stage_label,
            stage_label,
            Some(Box::new(FnCondLoad::new(cond_load))),
        );
        self
    }

    pub fn insert_stage_vec<L, B>(&self, prev: L, stages: Vec<B>) -> &Self
    where
        L: Into<Label>,
        B: Into<Label>,
    {
        self.inner.borrow_mut().insert_stage_vec(
            prev,
            stages
                .into_iter()
                .map(|stage| (stage, None))
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn insert_stage_vec_with_cond<L, B, Func, Args, Output>(
        &self,
        prev: L,
        stages: Vec<B>,
        cond_load: Func,
    ) -> &Self
    where
        L: Into<Label>,
        B: Into<Label>,
        Func: InjectOnce<Args, Output = Output> + Clone + 'static,
        Args: LocalProvide<LocalApp> + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>> + Send,
    {
        self.inner.borrow_mut().insert_stage_vec(
            prev,
            stages
                .into_iter()
                .map(|stage| {
                    (
                        stage,
                        Some(Box::new(FnCondLoad::new(cond_load.clone())) as BoxedLocalCondLoad),
                    )
                })
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn add_once_task<L, T, Args>(&self, stage_label: L, task: T) -> &Self
    where
        L: Into<Label>,
        T: IntoLocalOnceTaskDescriptor<Args> + 'static,
    {
        self.inner
            .borrow_mut()
            .add_once_task(stage_label, task.into_local_once_task_descriptor());
        self
    }

    pub async fn run(self, app: &LocalApp) -> Result<(), anyhow::Error> {
        ScheduleInner::run(mem::take(&mut self.inner.borrow_mut()), app).await
    }
}
