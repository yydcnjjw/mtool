use std::{collections::HashMap, mem, ops::DerefMut};

use anyhow::Context;
use async_recursion::async_recursion;
use futures::Future;
use minject::{InjectOnce, Provide};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use petgraph::{graph::NodeIndex, Direction, Graph};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use super::{CondLoad, FnCondLoad, IntoOnceTaskDescriptor, OnceTaskDescriptor, ScheduleGraph};
use crate::{app::App, label::Label};

enum Node {
    OnceTask(OnceTaskNode),
    Stage(StageNode),
}

struct OnceTaskNode {
    task: Mutex<Option<OnceTaskDescriptor>>,
    index: NodeIndex,
}

impl OnceTaskNode {
    fn new(task: OnceTaskDescriptor, idx: NodeIndex) -> Self {
        Self {
            task: Mutex::new(Some(task)),
            index: idx,
        }
    }

    async fn run_once(&self, app: &App) -> Result<(), anyhow::Error> {
        let task = { self.task.lock().await.take().context("task is not exist")? };
        task.run_once(app).await
    }
}

type BoxedCondLoad = Box<dyn CondLoad + Send + Sync>;

struct StageNode {
    index: NodeIndex,
    cond_load: Mutex<Option<BoxedCondLoad>>,
}

impl StageNode {
    fn new(idx: NodeIndex) -> Self {
        Self {
            index: idx,
            cond_load: Mutex::new(None),
        }
    }

    fn new_with_cond(idx: NodeIndex, cond_load: Option<BoxedCondLoad>) -> Self {
        Self {
            index: idx,
            cond_load: Mutex::new(cond_load),
        }
    }
}

type SchedGraph = Graph<Label, ()>;

#[derive(Default)]
pub struct ScheduleInner {
    graph: SchedGraph,
    node_index: HashMap<Label, Node>,
}

impl ScheduleInner {
    fn new() -> Self {
        let graph = SchedGraph::new();
        let node_index = HashMap::new();

        let mut this = Self { graph, node_index };

        let root = ScheduleGraph::Root.into();
        let stage = this.graph.add_node(root);
        this.node_index
            .insert(root, Node::Stage(StageNode::new(stage)));

        this
    }
}

impl ScheduleInner {
    fn insert_stage<L, B>(
        &mut self,
        prev_stage_label: L,
        stage_label: B,
        cond_load: Option<BoxedCondLoad>,
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
        stages: Vec<(B, Option<BoxedCondLoad>)>,
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

    fn add_once_task<L>(&mut self, label: L, task: OnceTaskDescriptor) -> &mut Self
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

    pub async fn run(mut self, app: &App) -> Result<(), anyhow::Error> {
        let root_stage = self.get_stage(ScheduleGraph::Root).unwrap();
        let neighbors = self
            .graph
            .neighbors_directed(root_stage.index, Direction::Outgoing);

        self.run_node_parallel(app, neighbors).await
    }

    async fn run_node_parallel<Iter>(&self, app: &App, iter: Iter) -> Result<(), anyhow::Error>
    where
        Iter: IntoIterator<Item = NodeIndex>,
    {
        let tasks = iter
            .into_iter()
            .map(|node| self.run_node(app, node))
            .collect::<Vec<_>>();

        futures::future::try_join_all(tasks).await.map(|_| ())
    }

    #[async_recursion]
    async fn run_node(&self, app: &App, index: NodeIndex) -> Result<(), anyhow::Error> {
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
                let need_run_task = match stage.cond_load.lock().await.take() {
                    Some(cond_load) => cond_load.load_with_cond(app).await?,
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

pub struct Schedule {
    inner: RwLock<ScheduleInner>,
    main_thread_loop: OnceCell<Box<dyn FnOnce() -> Result<(), anyhow::Error> + Send>>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(ScheduleInner::new()),
            main_thread_loop: OnceCell::new(),
        }
    }

    pub fn insert_stage<L, B>(&self, prev_stage_label: L, stage_label: B) -> &Self
    where
        L: Into<Label>,
        B: Into<Label>,
    {
        self.inner
            .write()
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
        Func: InjectOnce<Args, Output = Output> + Send + Sync + 'static,
        Args: Provide<App> + Send + Sync + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>> + Send,
    {
        self.inner.write().insert_stage(
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
        self.inner.write().insert_stage_vec(
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
        Func: InjectOnce<Args, Output = Output> + Send + Sync + Clone + 'static,
        Args: Provide<App> + Send + Sync + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>> + Send,
    {
        self.inner.write().insert_stage_vec(
            prev,
            stages
                .into_iter()
                .map(|stage| {
                    (
                        stage,
                        Some(Box::new(FnCondLoad::new(cond_load.clone())) as BoxedCondLoad),
                    )
                })
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn add_once_task<L, T, Args>(&self, stage_label: L, task: T) -> &Self
    where
        L: Into<Label>,
        T: IntoOnceTaskDescriptor<Args> + 'static,
    {
        self.inner
            .write()
            .add_once_task(stage_label, task.into_once_task_descriptor());
        self
    }

    pub fn setup_main_thread_loop<F>(&self, f: F) -> &Self
    where
        F: FnOnce() -> Result<(), anyhow::Error> + Send + 'static,
    {
        _ = self.main_thread_loop.set(Box::new(f));
        self
    }

    pub(crate) async fn run(mut self, app: &App) -> Result<(), anyhow::Error> {
        let tasks_schedule = mem::take(self.inner.write().deref_mut());
        let app = app.clone();
        let tasks_loop = tokio::spawn(async move {
            if let Err(e) = ScheduleInner::run(tasks_schedule, &app).await {
                warn!("{:?}", e);
            }
        });

        if let Some(thread_loop) = self.main_thread_loop.take() {
            tokio::task::block_in_place(move || thread_loop())?;
        }

        Ok(tasks_loop.await?)
    }
}
