use std::{collections::HashMap, fmt::{self, Debug}, ops::{Index, IndexMut}};
use petgraph::Direction;
use slotmap::{Key, KeyData, SecondaryMap, SlotMap, new_key_type};

use crate::schedule::{InternedTaskSet, condition::BoxCondition, task::ScheduleTask};
use super::dag::GraphNodeId;

new_key_type! {
    /// A unique identifier for a task in a [`ScheduleGraph`].
    pub struct TaskKey;
    /// A unique identifier for a task set in a [`ScheduleGraph`].
    pub struct TaskSetKey;
}

impl GraphNodeId for TaskKey {
    type Adjacent = (TaskKey, Direction);
    type Edge = (TaskKey, TaskKey);

    fn kind(&self) -> &'static str {
        "task"
    }
}

impl GraphNodeId for TaskSetKey {
    type Adjacent = (TaskSetKey, Direction);
    type Edge = (TaskSetKey, TaskSetKey);

    fn kind(&self) -> &'static str {
        "task set"
    }
}

impl TryFrom<NodeId> for TaskKey {
    type Error = TaskSetKey;

    fn try_from(value: NodeId) -> Result<Self, Self::Error> {
        match value {
            NodeId::Task(key) => Ok(key),
            NodeId::Set(key) => Err(key),
        }
    }
}

impl TryFrom<NodeId> for TaskSetKey {
    type Error = TaskKey;

    fn try_from(value: NodeId) -> Result<Self, Self::Error> {
        match value {
            NodeId::Task(key) => Err(key),
            NodeId::Set(key) => Ok(key),
        }
    }
}


pub enum Node {
    Task(TaskNode),
    TaskSet(TaskSet),
}

/// A [`TaskSet`] with metadata, stored in a [`ScheduleGraph`].
pub struct TaskSet {
    inner: InternedTaskSet,
}

impl TaskSet {
    pub fn new(set: InternedTaskSet) -> Self {
        Self { inner: set }
    }

    pub fn name(&self) -> String {
        format!("{:?}", &self.inner)
    }

    pub fn is_task_type(&self) -> bool {
        self.inner.task_type().is_some()
    }

    pub fn is_anonymous(&self) -> bool {
        self.inner.is_anonymous()
    }
}

pub struct TaskNode {
    inner: Option<ScheduleTask>,
}

impl TaskNode {
    /// Create a new [`TaskNode`]
    pub fn new(task: ScheduleTask) -> Self {
        Self { inner: Some(task) }
    }

    /// Obtain a reference to the [`ScheduleTask`] represented by this node.
    pub fn get(&self) -> Option<&ScheduleTask> {
        self.inner.as_ref()
    }

    /// Obtain a mutable reference to the [`ScheduleTask`] represented by this node.
    pub fn get_mut(&mut self) -> Option<&mut ScheduleTask> {
        self.inner.as_mut()
    }
}

#[derive(Default)]
pub struct TaskSets {
    sets: SlotMap<TaskSetKey, InternedTaskSet>,
    conditions: SecondaryMap<TaskSetKey, Vec<BoxCondition>>,
    ids: HashMap<InternedTaskSet, TaskSetKey>,
}

#[derive(Default)]
pub struct Tasks {
    tasks: SlotMap<TaskKey, TaskNode>,
    conditions: SecondaryMap<TaskKey, Vec<BoxCondition>>,
}

impl Tasks {
    /// Returns the number of tasks in this container.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Returns `true` if this container is empty.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns a reference to the task with the given key, if it exists.
    pub fn get(&self, key: TaskKey) -> Option<&ScheduleTask> {
        self.tasks.get(key).and_then(|node| node.get())
    }

    /// Returns a mutable reference to the task with the given key, if it exists.
    pub fn get_mut(&mut self, key: TaskKey) -> Option<&mut ScheduleTask> {
        self.tasks.get_mut(key).and_then(|node| node.get_mut())
    }

    /// Returns a mutable reference to the task with the given key. Will return
    /// `None` if the key does not exist.
    pub(crate) fn node_mut(&mut self, key: TaskKey) -> Option<&mut TaskNode> {
        self.tasks.get_mut(key)
    }

    /// Returns `true` if the task with the given key has conditions.
    pub fn has_conditions(&self, key: TaskKey) -> bool {
        self.conditions
            .get(key)
            .is_some_and(|conditions| !conditions.is_empty())
    }

    /// Returns a reference to the conditions for the task with the given key, if it exists.
    pub fn get_conditions(&self, key: TaskKey) -> Option<&[BoxCondition]> {
        self.conditions.get(key).map(Vec::as_slice)
    }

    /// Returns a mutable reference to the conditions for the task with the given key, if it exists.
    pub fn get_conditions_mut(&mut self, key: TaskKey) -> Option<&mut Vec<BoxCondition>> {
        self.conditions.get_mut(key)
    }

    /// Returns an iterator over all tasks and their conditions in this
    /// container.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (TaskKey, &ScheduleTask, &[BoxCondition])> + '_ {
        self.tasks.iter().filter_map(|(key, node)| {
            let task = node.get()?;
            let conditions = self
                .conditions
                .get(key)
                .map(Vec::as_slice)
                .unwrap_or_default();
            Some((key, task, conditions))
        })
    }

    /// Inserts a new task into the container, along with its conditions,
    /// and queues it to be initialized later in [`Tasks::initialize`].
    ///
    /// We have to defer initialization of tasks in the container until we have
    /// `&mut World` access, so we store these in a list until
    /// [`Tasks::initialize`] is called. This is usually done upon the first
    /// run of the schedule.
    pub fn insert(
        &mut self,
        task: ScheduleTask,
        conditions: Vec<BoxCondition>,
    ) -> TaskKey {
        let key = self.tasks.insert(TaskNode::new(task));
        self.conditions.insert(
            key,
            conditions
                .into_iter()
                .collect(),
        );
        // self.uninit.push(key);
        key
    }

    /// Remove a task with [`TaskKey`]
    pub(crate) fn remove(&mut self, key: TaskKey) -> bool {
        let mut found = false;
        if self.tasks.remove(key).is_some() {
            found = true;
        }

        if self.conditions.remove(key).is_some() {
            found = true;
        }

        // if let Some(index) = self.uninit.iter().position(|value| *value == key) {
        //     self.uninit.remove(index);
        //     found = true;
        // }

        found
    }

    /// Returns `true` if all tasks in this container have been initialized.
    // pub fn is_initialized(&self) -> bool {
    //     self.uninit.is_empty()
    // }

    /// Initializes all tasks and their conditions that have not been
    /// initialized yet.
    // pub fn initialize(&mut self, world: &mut World) {
    //     for key in self.uninit.drain(..) {
    //         let Some(task) = self.tasks.get_mut(key).and_then(|node| node.get_mut()) else {
    //             continue;
    //         };
    //         task.access = task.task.initialize(world);
    //         let Some(conditions) = self.conditions.get_mut(key) else {
    //             continue;
    //         };
    //         for condition in conditions {
    //             condition.access = condition.condition.initialize(world);
    //         }
    //     }
    // }

    /// Calculates the list of tasks that conflict with each other based on
    /// their access patterns.
    ///
    /// If the `Box<[ComponentId]>` is empty for a given pair of tasks, then the
    /// tasks conflict on [`World`] access in general (e.g. one of them is
    /// exclusive, or both tasks have `Query<EntityMut>`).
    // pub fn get_conflicting_tasks(
    //     &self,
    //     flat_dependency_analysis: &DagAnalysis<TaskKey>,
    //     flat_ambiguous_with: &UnGraphMap<TaskKey, ()>,
    //     ambiguous_with_all: &HashSet<NodeId>,
    //     // ignored_ambiguities: &BTreeSet<ComponentId>,
    // ) -> ConflictingTasks {
    //     let mut conflicting_tasks: Vec<(_, _, Box<[_]>)> = Vec::new();
    //     for &(a, b) in flat_dependency_analysis.disconnected() {
    //         if flat_ambiguous_with.contains_edge(a, b)
    //             || ambiguous_with_all.contains(&NodeId::Task(a))
    //             || ambiguous_with_all.contains(&NodeId::Task(b))
    //         {
    //             continue;
    //         }

    //         let task_a = &self[a];
    //         let task_b = &self[b];
    //         if task_a.is_exclusive() || task_b.is_exclusive() {
    //             conflicting_tasks.push((a, b, Box::new([])));
    //         } else {
    //             let access_a = &task_a.access;
    //             let access_b = &task_b.access;
    //             if !access_a.is_compatible(access_b) {
    //                 match access_a.get_conflicts(access_b) {
    //                     AccessConflicts::Individual(conflicts) => {
    //                         let conflicts: Box<[_]> = conflicts
    //                             .ones()
    //                             .map(ComponentId::get_sparse_set_index)
    //                             .filter(|id| !ignored_ambiguities.contains(id))
    //                             .collect();
    //                         if !conflicts.is_empty() {
    //                             conflicting_tasks.push((a, b, conflicts));
    //                         }
    //                     }
    //                     AccessConflicts::All => {
    //                         // there is no specific component conflicting, but the tasks are overall incompatible
    //                         // for example 2 tasks with `Query<EntityMut>`
    //                         conflicting_tasks.push((a, b, Box::new([])));
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     ConflictingTasks(conflicting_tasks)
    // }
}

impl Index<TaskKey> for Tasks {
    type Output = ScheduleTask;

    #[track_caller]
    fn index(&self, key: TaskKey) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("Task with key {:?} does not exist in the schedule", key))
    }
}

impl IndexMut<TaskKey> for Tasks {
    #[track_caller]
    fn index_mut(&mut self, key: TaskKey) -> &mut Self::Output {
        self.get_mut(key)
            .unwrap_or_else(|| panic!("Task with key {:?} does not exist in the schedule", key))
    }
}

/// Unique identifier for a task or task set stored in a [`ScheduleGraph`].
///
/// [`ScheduleGraph`]: crate::schedule::ScheduleGraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeId {
    Task(TaskKey),
    Set(TaskSetKey),
}

impl NodeId {
    /// Returns `true` if the identified node is a task.
    pub const fn is_task(&self) -> bool {
        matches!(self, NodeId::Task(_))
    }

    /// Returns `true` if the identified node is a system set.
    pub const fn is_set(&self) -> bool {
        matches!(self, NodeId::Set(_))
    }

    /// Returns the task key if the node is a task, otherwise `None`.
    pub const fn as_task(&self) -> Option<TaskKey> {
        match self {
            NodeId::Task(task) => Some(*task),
            NodeId::Set(_) => None,
        }
    }

    /// Returns the task set key if the node is a task set, otherwise `None`.
    pub const fn as_set(&self) -> Option<TaskSetKey> {
        match self {
            NodeId::Task(_) => None,
            NodeId::Set(set) => Some(*set),
        }
    }
}

impl GraphNodeId for NodeId {
    type Adjacent = CompactNodeIdAndDirection;
    type Edge = CompactNodeIdPair;

    fn kind(&self) -> &'static str {
        match self {
            NodeId::Task(n) => n.kind(),
            NodeId::Set(n) => n.kind(),
        }
    }
}

impl From<TaskKey> for NodeId {
    fn from(task: TaskKey) -> Self {
        NodeId::Task(task)
    }
}

impl From<TaskSetKey> for NodeId {
    fn from(set: TaskSetKey) -> Self {
        NodeId::Set(set)
    }
}

/// Compact storage of a [`NodeId`] and a [`Direction`].
#[derive(Clone, Copy)]
pub struct CompactNodeIdAndDirection {
    key: KeyData,
    is_task: bool,
    direction: Direction,
}

impl Debug for CompactNodeIdAndDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tuple: (_, _) = (*self).into();
        tuple.fmt(f)
    }
}

impl From<(NodeId, Direction)> for CompactNodeIdAndDirection {
    fn from((id, direction): (NodeId, Direction)) -> Self {
        let key = match id {
            NodeId::Task(key) => key.data(),
            NodeId::Set(key) => key.data(),
        };
        let is_task = id.is_task();

        Self {
            key,
            is_task,
            direction,
        }
    }
}

impl From<CompactNodeIdAndDirection> for (NodeId, Direction) {
    fn from(value: CompactNodeIdAndDirection) -> Self {
        let node = match value.is_task {
            true => NodeId::Task(value.key.into()),
            false => NodeId::Set(value.key.into()),
        };

        (node, value.direction)
    }
}

/// Compact storage of a [`NodeId`] pair.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct CompactNodeIdPair {
    key_a: KeyData,
    key_b: KeyData,
    is_task_a: bool,
    is_task_b: bool,
}

impl Debug for CompactNodeIdPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tuple: (_, _) = (*self).into();
        tuple.fmt(f)
    }
}

impl From<(NodeId, NodeId)> for CompactNodeIdPair {
    fn from((a, b): (NodeId, NodeId)) -> Self {
        let key_a = match a {
            NodeId::Task(index) => index.data(),
            NodeId::Set(index) => index.data(),
        };
        let is_task_a = a.is_task();

        let key_b = match b {
            NodeId::Task(index) => index.data(),
            NodeId::Set(index) => index.data(),
        };
        let is_task_b = b.is_task();

        Self {
            key_a,
            key_b,
            is_task_a,
            is_task_b,
        }
    }
}

impl From<CompactNodeIdPair> for (NodeId, NodeId) {
    fn from(value: CompactNodeIdPair) -> Self {
        let a = match value.is_task_a {
            true => NodeId::Task(value.key_a.into()),
            false => NodeId::Set(value.key_a.into()),
        };

        let b = match value.is_task_b {
            true => NodeId::Task(value.key_b.into()),
            false => NodeId::Set(value.key_b.into()),
        };

        (a, b)
    }
}


/// Pairs of tasks that conflict with each other along with the components
/// they conflict on, which prevents them from running in parallel. If the
/// component list is empty, the tasks conflict on [`World`] access in general
/// (e.g. one of them is exclusive, or both tasks have `Query<EntityMut>`).
#[derive(Clone, Debug, Default)]
pub struct ConflictingTasks(pub Vec<(TaskKey, TaskKey, // Box<[ComponentId]>
)>);

impl ConflictingTasks {
    /// Checks if there are any conflicting tasks, returning [`Ok`] if there
    /// are none, or an [`AmbiguousTaskConflictsWarning`] if there are.
    // pub fn check_if_not_empty(&self) -> Result<(), AmbiguousTaskConflictsWarning> {
    //     if self.0.is_empty() {
    //         Ok(())
    //     } else {
    //         Err(AmbiguousTaskConflictsWarning(self.clone()))
    //     }
    // }

    /// Converts the conflicting tasks into an iterator of their task names
    /// and the names of the components they conflict on.
    // pub fn to_string(
    //     &self,
    //     graph: &ScheduleGraph,
    //     components: &Components,
    // ) -> impl Iterator<Item = (String, String, Box<[DebugName]>)> {
    //     self.iter().map(move |(task_a, task_b, conflicts)| {
    //         let name_a = graph.get_node_name(&NodeId::Task(*task_a));
    //         let name_b = graph.get_node_name(&NodeId::Task(*task_b));

    //         let conflict_names: Box<[_]> = conflicts
    //             .iter()
    //             .map(|id| components.get_name(*id).unwrap())
    //             .collect();

    //         (name_a, name_b, conflict_names)
    //     })
    // }
}

// impl Deref for ConflictingTasks {
//     type Target = Vec<(TaskKey, TaskKey, Box<[ComponentId]>)>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
