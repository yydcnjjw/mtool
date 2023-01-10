use std::{collections::HashMap, mem, ops::DerefMut};

use anyhow::Context;
use async_recursion::async_recursion;
use petgraph::{graph::NodeIndex, Direction, Graph};
use tokio::sync::RwLock;

use crate::{define_label, App, IntoTaskDescriptor, Label, TaskDescriptor};

enum Node {
    Task(TaskNode),
    Stage(StageNode),
}

struct TaskNode {
    task: TaskDescriptor,
    index: NodeIndex,
}

impl TaskNode {
    fn new(task: TaskDescriptor, idx: NodeIndex) -> Self {
        Self { task, index: idx }
    }

    async fn run(&self, app: &App) -> Result<(), anyhow::Error> {
        self.task.run(app).await
    }
}

struct StageNode {
    index: NodeIndex,
}

impl StageNode {
    fn new(idx: NodeIndex) -> Self {
        Self { index: idx }
    }
}

type SchedGraph = Graph<Label, ()>;

define_label!(ScheduleGraph, Root);

#[derive(Default)]
pub struct ScheduleInner {
    graph: SchedGraph,
    node_index: HashMap<Label, Node>,
    root: NodeIndex,
}

impl ScheduleInner {
    fn new() -> Self {
        let mut graph = SchedGraph::new();
        let root = graph.add_node(ScheduleGraph::Root.into());

        Self {
            graph,
            node_index: HashMap::new(),
            root,
        }
    }
}

impl ScheduleInner {
    fn add_stage<L>(&mut self, stage_label: L) -> &mut Self
    where
        L: Into<Label>,
    {
        let stage_label = stage_label.into();

        let stage = self.graph.add_node(stage_label);

        self.graph.add_edge(self.root, stage, ());

        self.node_index
            .insert(stage_label, Node::Stage(StageNode::new(stage)));

        self
    }

    fn insert_stage<L, B>(&mut self, prev_stage_label: L, stage_label: B) -> &mut Self
    where
        L: Into<Label>,
        B: Into<Label>,
    {
        let stage_label = stage_label.into();

        let stage = self.graph.add_node(stage_label);

        let prev_stage = self.get_stage(prev_stage_label).unwrap();
        self.graph.add_edge(prev_stage.index, stage, ());

        self.node_index
            .insert(stage_label, Node::Stage(StageNode::new(stage)));

        self
    }

    fn add_task<L>(&mut self, label: L, task: TaskDescriptor) -> &mut Self
    where
        L: Into<Label>,
    {
        let label = label.into();
        let index = self
            .get_node_index(label.clone())
            .expect(&format!("{}", &label));

        let task_label = task.label;

        let task_node = self.graph.add_node(task_label);

        self.graph.add_edge(index, task_node, ());

        self.node_index
            .insert(task_label, Node::Task(TaskNode::new(task, task_node)));

        self
    }

    fn get_stage<L>(&self, stage_label: L) -> Option<&StageNode>
    where
        L: Into<Label>,
    {
        self.node_index
            .get(&stage_label.into())
            .and_then(|v| match v {
                Node::Task(_) => None,
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
            Node::Task(t) => Some(t.index),
            Node::Stage(s) => Some(s.index),
        })
    }

    pub async fn run(self, app: &App) -> Result<(), anyhow::Error> {
        self.run_node_parallel(
            app,
            self.graph
                .neighbors_directed(self.root, Direction::Outgoing),
        )
        .await
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
            Node::Task(task) => {
                if let Some(cond) = &task.task.cond_load {
                    if cond.load_with_cond(app).await? {
                        task.run(app)
                            .await
                            .context(format!("running task: {}", label))?;
                    }
                } else {
                    task.run(app)
                        .await
                        .context(format!("running task: {}", label))?;
                }

                self.run_node_parallel(
                    app,
                    self.graph.neighbors_directed(index, Direction::Outgoing),
                )
                .await?;
            }
            Node::Stage(_) => {
                log::debug!("run stage: {}", label);
                self.run_node_parallel(
                    app,
                    self.graph
                        .neighbors_directed(index, Direction::Outgoing)
                        .filter(|index| matches!(self.get_node_with_index(*index), Node::Task(_))),
                )
                .await?;

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
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(ScheduleInner::new()),
        }
    }

    pub async fn add_stage<L>(&self, stage_label: L) -> &Self
    where
        L: Into<Label>,
    {
        self.inner.write().await.add_stage(stage_label);
        self
    }

    pub async fn insert_stage<L, B>(&self, prev_stage_label: L, stage_label: B) -> &Self
    where
        L: Into<Label>,
        B: Into<Label>,
    {
        self.inner
            .write()
            .await
            .insert_stage(prev_stage_label, stage_label);
        self
    }

    pub async fn add_task<L, T, Args>(&self, stage_label: L, task: T) -> &Self
    where
        L: Into<Label>,
        T: IntoTaskDescriptor<Args> + 'static,
    {
        self.inner
            .write()
            .await
            .add_task(stage_label, task.into_task_descriptor());
        self
    }

    pub async fn run(self, app: &App) -> Result<(), anyhow::Error> {
        ScheduleInner::run(mem::take(self.inner.write().await.deref_mut()), app).await
    }
}
