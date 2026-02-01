use std::collections::HashMap;

use disqualified::ShortName;
use fixedbitset::FixedBitSet;
use petgraph::{dot::Dot, prelude::*};

use crate::context::Context;

use super::{
    InternedScheduleLabel, InternedTaskSet,
    condition::BoxCondition,
    config::{Chain, IntoScheduleConfigs, Schedulable, ScheduleConfig, ScheduleConfigs},
    error::ScheduleBuildError,
    graph_info::{Ambiguity, Dependency, DependencyKind, GraphInfo},
    task::ScheduleTask,
};

#[derive(Default)]
pub struct ScheduleGraph {
    tasks: Vec<TaskNode>,

    task_conditions: Vec<Vec<BoxCondition>>,

    task_sets: Vec<TaskSetNode>,

    task_set_conditions: Vec<Vec<BoxCondition>>,

    task_set_ids: HashMap<InternedTaskSet, NodeId>,

    uninit: Vec<(NodeId, usize)>,

    hierarchy: Dag,

    dependency: Dag,
}

impl ScheduleGraph {
    fn process_config<T: ProcessScheduleConfig + Schedulable>(
        &mut self,
        config: ScheduleConfig<T>,
        collect_nodes: bool,
    ) -> ProcessConfigsResult {
        ProcessConfigsResult {
            densely_chained: true,
            nodes: collect_nodes
                .then_some(T::process_config(self, config))
                .into_iter()
                .collect(),
        }
    }

    fn apply_collective_conditions<T>(
        &mut self,
        configs: &mut [ScheduleConfigs<T>],
        collective_conditions: Vec<BoxCondition>,
    ) where
        T: ProcessScheduleConfig + Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>,
    {
        if !collective_conditions.is_empty() {
            // TODO:
            // if let [config] = configs {
            //     for condition in collective_conditions {
            //         config.run_if_dyn(condition);
            //     }
            // } else {
            //     let set = self.create_anonymous_set();
            //     for config in configs.iter_mut() {
            //         config.in_set_inner(set.intern());
            //     }
            //     let mut set_config = InternedSystemSet::into_config(set.intern());
            //     set_config.conditions.extend(collective_conditions);
            //     self.configure_set_inner(set_config).unwrap();
            // }
        }
    }

    fn process_configs<T>(
        &mut self,
        configs: ScheduleConfigs<T>,
        collect_nodes: bool,
    ) -> ProcessConfigsResult
    where
        T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain> + ProcessScheduleConfig,
    {
        match configs {
            ScheduleConfigs::ScheduleConfig(config) => self.process_config(config, collect_nodes),
            ScheduleConfigs::Configs {
                configs,
                collective_conditions: _,
                metadata,
            } => {
                // TODO:
                // self.apply_collective_conditions(&mut configs, collective_conditions);

                let is_chained = matches!(metadata, Chain::Chained(_));
                // Densely chained if
                // * chained and all configs in the chain are densely chained, or
                // * unchained with a single densely chained config
                let mut densely_chained = is_chained || configs.len() == 1;
                let mut configs = configs.into_iter();
                let mut nodes = Vec::new();

                let Some(first) = configs.next() else {
                    return ProcessConfigsResult {
                        nodes: Vec::new(),
                        densely_chained,
                    };
                };
                let mut previous_result = self.process_configs(first, collect_nodes || is_chained);
                densely_chained &= previous_result.densely_chained;

                for current in configs {
                    let current_result = self.process_configs(current, collect_nodes || is_chained);
                    densely_chained &= current_result.densely_chained;

                    // TODO: why
                    if let Chain::Chained(_chain_options) = &metadata {
                        // if the current result is densely chained, we only need to chain the first node
                        let current_nodes = if current_result.densely_chained {
                            &current_result.nodes[..1]
                        } else {
                            &current_result.nodes
                        };
                        // if the previous result was densely chained, we only need to chain the last node
                        let previous_nodes = if previous_result.densely_chained {
                            &previous_result.nodes[previous_result.nodes.len() - 1..]
                        } else {
                            &previous_result.nodes
                        };

                        for previous_node in previous_nodes {
                            for current_node in current_nodes {
                                self.dependency
                                    .graph
                                    .add_edge(*previous_node, *current_node, ());

                                // TODO:
                                // for pass in self.passes.values_mut() {
                                //     pass.add_dependency(
                                //         *previous_node,
                                //         *current_node,
                                //         chain_options,
                                //     );
                                // }
                            }
                        }
                    }
                    if collect_nodes {
                        nodes.append(&mut previous_result.nodes);
                    }

                    previous_result = current_result;
                }
                if collect_nodes {
                    nodes.append(&mut previous_result.nodes);
                }

                ProcessConfigsResult {
                    nodes,
                    densely_chained,
                }
            }
        }
    }

    fn add_task_inner(
        &mut self,
        config: ScheduleConfig<ScheduleTask>,
    ) -> Result<NodeId, ScheduleBuildError> {
        let id = NodeId::Task(self.tasks.len());

        // graph updates are immediate
        self.update_graphs(id, config.metadata)?;

        // system init has to be deferred (need `&mut World`)
        self.uninit.push((id, 0));
        self.tasks.push(TaskNode::new(config.node));
        self.task_conditions.push(config.conditions);

        Ok(id)
    }

    fn update_graphs(
        &mut self,
        id: NodeId,
        graph_info: GraphInfo,
    ) -> Result<(), ScheduleBuildError> {
        self.check_hierarchy_sets(&id, &graph_info)?;
        self.check_edges(&id, &graph_info)?;
        // TODO:
        // self.changed = true;

        let GraphInfo {
            hierarchy: sets,
            dependencies,
            ambiguous_with: _,
            ..
        } = graph_info;

        self.hierarchy.graph.add_node(id);
        self.dependency.graph.add_node(id);

        for set in sets.into_iter().map(|set| self.task_set_ids[&set]) {
            self.hierarchy.graph.add_edge(set, id, ());

            // ensure set also appears in dependency graph
            self.dependency.graph.add_node(set);
        }

        for (kind, set, _options) in dependencies
            .into_iter()
            .map(|Dependency { kind, set, options }| (kind, self.task_set_ids[&set], options))
        {
            let (lhs, rhs) = match kind {
                DependencyKind::Before => (id, set),
                DependencyKind::After => (set, id),
            };
            self.dependency.graph.add_edge(lhs, rhs, ());

            // TODO:
            // for pass in self.passes.values_mut() {
            //     pass.add_dependency(lhs, rhs, &options);
            // }

            // ensure set also appears in hierarchy graph
            self.hierarchy.graph.add_node(set);
        }

        // TODO:
        // match ambiguous_with {
        //     Ambiguity::Check => (),
        //     Ambiguity::IgnoreWithSet(ambiguous_with) => {
        //         for set in ambiguous_with
        //             .into_iter()
        //             .map(|set| self.task_set_ids[&set])
        //         {
        //             self.ambiguous_with.add_edge(id, set);
        //         }
        //     }
        //     Ambiguity::IgnoreAll => {
        //         self.ambiguous_with_all.insert(id);
        //     }
        // }

        Ok(())
    }

    #[inline]
    fn get_node_name(&self, id: &NodeId) -> String {
        ShortName(&match id {
            NodeId::Task(_) => self.tasks[id.index()].get().unwrap().name().to_string(),
            NodeId::Set(_) => {
                let set = &self.task_sets[id.index()];
                if set.is_anonymous() {
                    todo!()
                } else {
                    set.name()
                }
            }
        })
        .to_string()
    }

    #[track_caller]
    fn configure_sets<M>(&mut self, sets: impl IntoScheduleConfigs<InternedTaskSet, M>) {
        self.process_configs(sets.into_configs(), false);
    }

    fn configure_set_inner(
        &mut self,
        set: ScheduleConfig<InternedTaskSet>,
    ) -> Result<NodeId, ScheduleBuildError> {
        let ScheduleConfig {
            node: set,
            metadata,
            mut conditions,
        } = set;

        let id = match self.task_set_ids.get(&set) {
            Some(&id) => id,
            None => self.add_task_set(set),
        };

        // graph updates are immediate
        self.update_graphs(id, metadata)?;

        // task init has to be deferred (need `&mut World`)
        let task_set_conditions = &mut self.task_set_conditions[id.index()];
        self.uninit.push((id, task_set_conditions.len()));
        task_set_conditions.append(&mut conditions);

        Ok(id)
    }

    fn add_task_set(&mut self, set: InternedTaskSet) -> NodeId {
        let id = NodeId::Set(self.task_sets.len());
        self.task_sets.push(TaskSetNode::new(set));
        self.task_set_conditions.push(Vec::new());
        self.task_set_ids.insert(set, id);
        id
    }

    /// Checks that a task set isn't included in itself.
    /// If not present, add the set to the graph.
    fn check_hierarchy_set(
        &mut self,
        id: &NodeId,
        set: InternedTaskSet,
    ) -> Result<(), ScheduleBuildError> {
        match self.task_set_ids.get(&set) {
            Some(set_id) => {
                if id == set_id {
                    return Err(ScheduleBuildError::HierarchyLoop {
                        task_set: self.get_node_name(id),
                    });
                }
            }
            None => {
                self.add_task_set(set);
            }
        }

        Ok(())
    }

    /// Check that no set is included in itself.
    /// Add all the sets from the [`GraphInfo`]'s hierarchy to the graph.
    fn check_hierarchy_sets(
        &mut self,
        id: &NodeId,
        graph_info: &GraphInfo,
    ) -> Result<(), ScheduleBuildError> {
        for &set in &graph_info.hierarchy {
            self.check_hierarchy_set(id, set)?;
        }

        Ok(())
    }

    /// Checks that no task set is dependent on itself.
    /// Add all the sets from the [`GraphInfo`]'s dependencies to the graph.
    fn check_edges(
        &mut self,
        id: &NodeId,
        graph_info: &GraphInfo,
    ) -> Result<(), ScheduleBuildError> {
        for Dependency { set, .. } in &graph_info.dependencies {
            match self.task_set_ids.get(set) {
                Some(set_id) => {
                    if id == set_id {
                        return Err(ScheduleBuildError::DependencyLoop {
                            task_set: self.get_node_name(id),
                        });
                    }
                }
                None => {
                    self.add_task_set(*set);
                }
            }
        }

        // TODO:
        // if let Ambiguity::IgnoreWithSet(ambiguous_with) = &graph_info.ambiguous_with {
        //     for set in ambiguous_with {
        //         if !self.task_set_ids.contains_key(set) {
        //             self.add_task_set(*set);
        //         }
        //     }
        // }

        Ok(())
    }

    pub fn topsort_graph(
        &self,
        graph: &DiGraph,
        // report: ReportCycles,
    ) -> Result<Vec<NodeId>, ScheduleBuildError> {
        // Tarjan's SCC algorithm returns elements in *reverse* topological order.
        let mut top_sorted_nodes = Vec::with_capacity(graph.node_count());
        let mut sccs_with_cycles = Vec::new();

        for scc in petgraph::algo::scc::tarjan_scc(graph) {
            // A strongly-connected component is a group of nodes who can all reach each other
            // through one or more paths. If an SCC contains more than one node, there must be
            // at least one cycle within them.
            top_sorted_nodes.extend_from_slice(&scc);
            if scc.len() > 1 {
                sccs_with_cycles.push(scc);
            }
        }

        if sccs_with_cycles.is_empty() {
            // reverse to get topological order
            top_sorted_nodes.reverse();
            Ok(top_sorted_nodes)
        } else {
            // let mut cycles = Vec::new();
            // for scc in &sccs_with_cycles {
            //     cycles.append(&mut simple_cycles_in_component(graph, scc));
            // }

            // let error = match report {
            //     ReportCycles::Hierarchy => ScheduleBuildError::HierarchyCycle(
            //         self.get_hierarchy_cycles_error_message(&cycles),
            //     ),
            //     ReportCycles::Dependency => ScheduleBuildError::DependencyCycle(
            //         self.get_dependency_cycles_error_message(&cycles),
            //     ),
            // };

            // Err(error)

            unimplemented!()
        }
    }

    /// Return a map from task set `NodeId` to a list of task `NodeId`s that are included in the set.
    /// Also return a map from task set `NodeId` to a `FixedBitSet` of task `NodeId`s that are included in the set,
    /// where the bitset order is the same as `self.tasks`
    fn map_sets_to_tasks(
        &self,
        hierarchy_topsort: &[NodeId],
        hierarchy_graph: &DiGraph,
    ) -> (HashMap<NodeId, Vec<NodeId>>, HashMap<NodeId, FixedBitSet>) {
        let mut set_tasks: HashMap<NodeId, Vec<NodeId>> =
            HashMap::with_capacity_and_hasher(self.task_sets.len(), Default::default());
        let mut set_task_bitsets =
            HashMap::with_capacity_and_hasher(self.task_sets.len(), Default::default());
        for &id in hierarchy_topsort.iter().rev() {
            if id.is_task() {
                continue;
            }

            let mut tasks = Vec::new();
            let mut task_bitset = FixedBitSet::with_capacity(self.tasks.len());

            for child in hierarchy_graph.neighbors_directed(id, Outgoing) {
                match child {
                    NodeId::Task(_) => {
                        tasks.push(child);
                        task_bitset.insert(child.index());
                    }
                    NodeId::Set(_) => {
                        let child_tasks = set_tasks.get(&child).unwrap();
                        let child_task_bitset = set_task_bitsets.get(&child).unwrap();
                        tasks.extend_from_slice(child_tasks);
                        task_bitset.union_with(child_task_bitset);
                    }
                }
            }

            set_tasks.insert(id, tasks);
            set_task_bitsets.insert(id, task_bitset);
        }
        (set_tasks, set_task_bitsets)
    }

    fn get_dependency_flattened(&mut self, set_tasks: &HashMap<NodeId, Vec<NodeId>>) -> DiGraph {
        // flatten: combine `in_set` with `before` and `after` information
        // have to do it like this to preserve transitivity
        let mut dependency_flattened = self.dependency.graph.clone();
        let mut temp = Vec::new();
        for (&set, tasks) in set_tasks {
            // TODO:
            // for pass in self.passes.values_mut() {
            //     pass.collapse_set(set, tasks, &dependency_flattened, &mut temp);
            // }
            if tasks.is_empty() {
                // collapse dependencies for empty sets
                for a in dependency_flattened.neighbors_directed(set, Incoming) {
                    for b in dependency_flattened.neighbors_directed(set, Outgoing) {
                        temp.push((a, b));
                    }
                }
            } else {
                for a in dependency_flattened.neighbors_directed(set, Incoming) {
                    for &sys in tasks {
                        temp.push((a, sys));
                    }
                }

                for b in dependency_flattened.neighbors_directed(set, Outgoing) {
                    for &sys in tasks {
                        temp.push((sys, b));
                    }
                }
            }

            dependency_flattened.remove_node(set);
            for (a, b) in temp.drain(..) {
                dependency_flattened.add_edge(a, b, ());
            }
        }

        dependency_flattened
    }

    /// Build a [`SystemSchedule`] optimized for scheduler access from the [`ScheduleGraph`].
    ///
    /// This method also
    /// - checks for dependency or hierarchy cycles
    /// - checks for system access conflicts and reports ambiguities
    pub fn build_schedule(
        &mut self,
        // ctx: &Context,
        // schedule_label: InternedScheduleLabel,
        // ignored_ambiguities: &BTreeSet<ComponentId>,
    ) -> Result<(), ScheduleBuildError> {
        self.hierarchy.topsort = self.topsort_graph(&self.hierarchy.graph)?;

        self.dependency.topsort = self.topsort_graph(&self.dependency.graph)?;

        let (set_systems, set_system_bitsets) =
            self.map_sets_to_tasks(&self.hierarchy.topsort, &self.hierarchy.graph);

        let mut dependency_flattened = self.get_dependency_flattened(&set_systems);

        let mut dependency_flattened_dag = Dag {
            topsort: self.topsort_graph(&dependency_flattened).unwrap(),
            graph: dependency_flattened,
        };

        println!(
            "{:?}",
            dependency_flattened_dag
                .topsort
                .iter()
                .map(|node| self.get_node_name(node))
                .collect::<Vec<_>>()
        );

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &dependency_flattened_dag.graph,
                &[],
                &|g, edge| { r#"label = """#.to_string()  },
                &|g, node| { format!(r#"label = "{}""#, self.get_node_name(&node.0)) }
            )
        );
        Ok(())

        // // check hierarchy for cycles
        // self.hierarchy.topsort =
        //     self.topsort_graph(&self.hierarchy.graph, ReportCycles::Hierarchy)?;

        // let hier_results = check_graph(&self.hierarchy.graph, &self.hierarchy.topsort);
        // self.optionally_check_hierarchy_conflicts(&hier_results.transitive_edges, schedule_label)?;

        // // remove redundant edges
        // self.hierarchy.graph = hier_results.transitive_reduction;

        // // check dependencies for cycles
        // self.dependency.topsort =
        //     self.topsort_graph(&self.dependency.graph, ReportCycles::Dependency)?;

        // // check for systems or system sets depending on sets they belong to
        // let dep_results = check_graph(&self.dependency.graph, &self.dependency.topsort);
        // self.check_for_cross_dependencies(&dep_results, &hier_results.connected)?;

        // // map all system sets to their systems
        // // go in reverse topological order (bottom-up) for efficiency
        // let (set_systems, set_system_bitsets) =
        //     self.map_sets_to_systems(&self.hierarchy.topsort, &self.hierarchy.graph);
        // self.check_order_but_intersect(&dep_results.connected, &set_system_bitsets)?;

        // // check that there are no edges to system-type sets that have multiple instances
        // self.check_system_type_set_ambiguity(&set_systems)?;

        // let mut dependency_flattened = self.get_dependency_flattened(&set_systems);

        // // modify graph with build passes
        // let mut passes = core::mem::take(&mut self.passes);
        // for pass in passes.values_mut() {
        //     pass.build(world, self, &mut dependency_flattened)?;
        // }
        // self.passes = passes;

        // // topsort
        // let mut dependency_flattened_dag = Dag {
        //     topsort: self.topsort_graph(&dependency_flattened, ReportCycles::Dependency)?,
        //     graph: dependency_flattened,
        // };

        // let flat_results = check_graph(
        //     &dependency_flattened_dag.graph,
        //     &dependency_flattened_dag.topsort,
        // );

        // // remove redundant edges
        // dependency_flattened_dag.graph = flat_results.transitive_reduction;

        // // flatten: combine `in_set` with `ambiguous_with` information
        // let ambiguous_with_flattened = self.get_ambiguous_with_flattened(&set_systems);

        // // check for conflicts
        // let conflicting_systems = self.get_conflicting_systems(
        //     &flat_results.disconnected,
        //     &ambiguous_with_flattened,
        //     ignored_ambiguities,
        // );
        // self.optionally_check_conflicts(&conflicting_systems, world.components(), schedule_label)?;
        // self.conflicting_systems = conflicting_systems;

        // // build the schedule
        // Ok(self.build_schedule_inner(dependency_flattened_dag, hier_results.reachable))
    }

    fn dot(&self) {
        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.hierarchy.graph,
                &[],
                &|g, edge| { String::new() },
                &|g, node| { format!(r#"label = "{}""#, self.get_node_name(&node.0)) }
            )
        );

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.dependency.graph,
                &[],
                &|g, edge| { String::new() },
                &|g, node| { format!(r#"label = "{}""#, self.get_node_name(&node.0)) }
            )
        );
    }
}

enum Node {
    Task(TaskNode),
    TaskSet(TaskSetNode),
}

/// A [`TaskSet`] with metadata, stored in a [`ScheduleGraph`].
struct TaskSetNode {
    inner: InternedTaskSet,
}

impl TaskSetNode {
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

type DiGraph = DiGraphMap<NodeId, ()>;

#[derive(Default)]
struct Dag {
    graph: DiGraph,
    topsort: Vec<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeId {
    Task(usize),
    Set(usize),
}

impl NodeId {
    /// Returns the internal integer value.
    pub const fn index(&self) -> usize {
        match self {
            NodeId::Task(index) | NodeId::Set(index) => *index,
        }
    }

    /// Returns `true` if the identified node is a task.
    pub const fn is_task(&self) -> bool {
        matches!(self, NodeId::Task(_))
    }

    /// Returns `true` if the identified node is a system set.
    pub const fn is_set(&self) -> bool {
        matches!(self, NodeId::Set(_))
    }

    /// Compare this [`NodeId`] with another.
    pub const fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        use NodeId::{Set, Task};
        use core::cmp::Ordering::{Equal, Greater, Less};

        match (self, other) {
            (Task(a), Task(b)) | (Set(a), Set(b)) => match a.checked_sub(*b) {
                None => Less,
                Some(0) => Equal,
                Some(_) => Greater,
            },
            (Task(_), Set(_)) => Less,
            (Set(_), Task(_)) => Greater,
        }
    }
}

/// Values returned by [`ScheduleGraph::process_configs`]
struct ProcessConfigsResult {
    /// All nodes contained inside this `process_configs` call's [`ScheduleConfigs`] hierarchy,
    /// if `ancestor_chained` is true
    nodes: Vec<NodeId>,
    /// TODO: why
    /// True if and only if all nodes are "densely chained", meaning that all nested nodes
    /// are linearly chained (as if `after` task ordering had been applied between each node)
    /// in the order they are defined
    densely_chained: bool,
}

/// Trait used by [`ScheduleGraph::process_configs`] to process a single [`ScheduleConfig`].
trait ProcessScheduleConfig: Schedulable + Sized {
    /// Process a single [`ScheduleConfig`].
    fn process_config(schedule_graph: &mut ScheduleGraph, config: ScheduleConfig<Self>) -> NodeId;
}

impl ProcessScheduleConfig for ScheduleTask {
    fn process_config(schedule_graph: &mut ScheduleGraph, config: ScheduleConfig<Self>) -> NodeId {
        schedule_graph.add_task_inner(config).unwrap()
    }
}

impl ProcessScheduleConfig for InternedTaskSet {
    fn process_config(schedule_graph: &mut ScheduleGraph, config: ScheduleConfig<Self>) -> NodeId {
        schedule_graph.configure_set_inner(config).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use petgraph::dot::Dot;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_graph() {
        let mut graph = ScheduleGraph::default();

        #[derive(TaskSet, Debug, Hash, Clone, PartialEq, Eq)]
        enum TestSet {
            PreTest,
            Test,
            PostTest,
        }

        async fn run_test1() -> Result<(), ContextError> {
            Ok(())
        }

        async fn run_test2() -> Result<(), ContextError> {
            Ok(())
        }

        async fn run_test3() -> Result<(), ContextError> {
            Ok(())
        }

        async fn pre_test() -> Result<(), ContextError> {
            Ok(())
        }

        async fn post_test() -> Result<(), ContextError> {
            Ok(())
        }

        graph.configure_sets((TestSet::PreTest, TestSet::Test, TestSet::PostTest).chain());

        graph.process_configs(
            (run_test1, run_test2, run_test3)
                // .chain()
                .in_set(TestSet::Test),
            false,
        );
        graph.process_configs(pre_test.in_set(TestSet::PreTest), false);
        graph.process_configs(post_test.in_set(TestSet::PostTest), false);

        graph.dot();

        graph.build_schedule().unwrap();
    }
}
