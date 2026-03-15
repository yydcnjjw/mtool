use std::collections::HashMap;

use disqualified::ShortName;
use fixedbitset::FixedBitSet;
use petgraph::{dot::Dot, prelude::*};

use crate::context::Context;

use super::{
    InternedScheduleLabel, InternedTaskSet,
    condition::BoxCondition,
    config::{Chain, IntoScheduleConfigs, Schedulable, ScheduleConfig, ScheduleConfigs},
    dag::Dag,
    error::ScheduleBuildError,
    graph_info::{Ambiguity, Dependency, DependencyKind, GraphInfo},
    node::{NodeId, TaskKey, TaskSetKey, TaskSets, Tasks},
    task::ScheduleTask,
};

#[derive(Default)]
pub struct ScheduleGraph {
    tasks: Tasks,
    task_sets: TaskSets,
    hierarchy: Dag<NodeId>,
    dependency: Dag<NodeId>,
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

    /// Builds an execution-optimized [`SystemSchedule`] from the current state
    /// of the graph. Also returns any warnings that were generated during the
    /// build process.
    ///
    /// This method also
    /// - checks for dependency or hierarchy cycles
    /// - checks for system access conflicts and reports ambiguities
    pub fn build_schedule(
        &mut self,
        // world: &mut World,
        // ignored_ambiguities: &BTreeSet<ComponentId>,
    ) // -> Result<(SystemSchedule, Vec<ScheduleBuildWarning>), ScheduleBuildError>
 { // 
        let mut warnings = Vec::new();

        // Check system set memberships for cycles.
        let hierarchy_analysis = self
            .hierarchy
            .analyze()
            .map_err(ScheduleBuildError::HierarchySort)?;

        // Check for redundant system set memberships, logging warnings or
        // returning errors as configured.
        if self.settings.hierarchy_detection != LogLevel::Ignore
            && let Err(e) = hierarchy_analysis.check_for_redundant_edges()
        {
            match self.settings.hierarchy_detection {
                LogLevel::Error => return Err(ScheduleBuildWarning::HierarchyRedundancy(e).into()),
                LogLevel::Warn => warnings.push(ScheduleBuildWarning::HierarchyRedundancy(e)),
                LogLevel::Ignore => unreachable!(),
            }
        }
        // Remove redundant system set memberships.
        self.hierarchy.remove_redundant_edges(&hierarchy_analysis);

        // Check system and system set ordering dependencies for cycles.
        let dependency_analysis = self
            .dependency
            .analyze()
            .map_err(ScheduleBuildError::DependencySort)?;

        // System sets that share systems and have an ordering dependency cannot be ordered.
        dependency_analysis.check_for_cross_dependencies(&hierarchy_analysis)?;

        // Group all systems by the system sets they belong to.
        self.set_systems = self
            .hierarchy
            .group_by_key(self.system_sets.len())
            .map_err(ScheduleBuildError::HierarchySort)?;
        // Check for system sets that share systems but have an ordering dependency.
        dependency_analysis.check_for_overlapping_groups(&self.set_systems)?;

        // There can be no edges to system-type sets that have multiple instances.
        self.system_sets.check_type_set_ambiguity(
            &self.set_systems,
            &self.ambiguous_with,
            &self.dependency,
        )?;

        // Flatten system ordering dependencies by collapsing system sets. This
        // means that if a system set has ordering dependencies, those
        // dependencies are applied to all systems in the set.
        let mut flat_dependency =
            self.set_systems
                .flatten(self.dependency.clone(), |set, systems, flattening, temp| {
                    for pass in self.passes.values_mut() {
                        pass.collapse_set(set, systems, flattening, temp);
                    }
                });

        // Allow modification of the schedule graph by build passes.
        let mut passes = core::mem::take(&mut self.passes);
        for pass in passes.values_mut() {
            pass.build(world, self, &mut flat_dependency)?;
        }
        self.passes = passes;

        // Check system ordering dependencies for cycles after collapsing sets
        // and applying build passes.
        let flat_dependency_analysis = flat_dependency
            .analyze()
            .map_err(ScheduleBuildError::FlatDependencySort)?;
        flat_dependency.remove_redundant_edges(&flat_dependency_analysis);

        // Flatten accepted system ordering ambiguities by collapsing system sets.
        // This means that if a system set is allowed to have ambiguous ordering
        // with another set, all systems in the first set are allowed to have
        // ambiguous ordering with all systems in the second set.
        let flat_ambiguous_with = self.set_systems.flatten_undirected(&self.ambiguous_with);

        // Find all system ordering ambiguities, ignoring those that are accepted.
        self.conflicting_systems = self.systems.get_conflicting_systems(
            &flat_dependency_analysis,
            &flat_ambiguous_with,
            &self.ambiguous_with_all,
            ignored_ambiguities,
        );
        // If there are any ambiguities, log warnings or return errors as configured.
        if self.settings.ambiguity_detection != LogLevel::Ignore
            && let Err(e) = self.conflicting_systems.check_if_not_empty()
        {
            match self.settings.ambiguity_detection {
                LogLevel::Error => return Err(ScheduleBuildWarning::Ambiguity(e).into()),
                LogLevel::Warn => warnings.push(ScheduleBuildWarning::Ambiguity(e)),
                LogLevel::Ignore => unreachable!(),
            }
        }

        // build the schedule
        Ok((
            self.build_schedule_inner(flat_dependency, hierarchy_analysis),
            warnings,
        ))
    }

    fn build_schedule_inner(
        &self,
        flat_dependency: Dag<SystemKey>,
        hierarchy_analysis: DagAnalysis<NodeId>,
    ) -> SystemSchedule {
        let dg_system_ids = flat_dependency.get_toposort().unwrap().to_vec();
        let dg_system_idx_map = dg_system_ids
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect::<HashMap<_, _>>();

        let hierarchy_toposort = self.hierarchy.get_toposort().unwrap();
        let hg_systems = hierarchy_toposort
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, id)| Some((i, id.as_system()?)))
            .collect::<Vec<_>>();
        let (hg_set_with_conditions_idxs, hg_set_ids): (Vec<_>, Vec<_>) = hierarchy_toposort
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, id)| {
                // ignore system sets that have no conditions
                // ignore system type sets (already covered, they don't have conditions)
                let key = id.as_set()?;
                self.system_sets.has_conditions(key).then_some((i, key))
            })
            .unzip();

        let sys_count = self.systems.len();
        let set_with_conditions_count = hg_set_ids.len();
        let hg_node_count = self.hierarchy.node_count();

        // get the number of dependencies and the immediate dependents of each system
        // (needed by multi_threaded executor to run systems in the correct order)
        let mut system_dependencies = Vec::with_capacity(sys_count);
        let mut system_dependents = Vec::with_capacity(sys_count);
        for &sys_key in &dg_system_ids {
            let num_dependencies = flat_dependency
                .neighbors_directed(sys_key, Incoming)
                .count();

            let dependents = flat_dependency
                .neighbors_directed(sys_key, Outgoing)
                .map(|dep_id| dg_system_idx_map[&dep_id])
                .collect::<Vec<_>>();

            system_dependencies.push(num_dependencies);
            system_dependents.push(dependents);
        }

        // get the rows and columns of the hierarchy graph's reachability matrix
        // (needed to we can evaluate conditions in the correct order)
        let mut systems_in_sets_with_conditions =
            vec![FixedBitSet::with_capacity(sys_count); set_with_conditions_count];
        for (i, &row) in hg_set_with_conditions_idxs.iter().enumerate() {
            let bitset = &mut systems_in_sets_with_conditions[i];
            for &(col, sys_key) in &hg_systems {
                let idx = dg_system_idx_map[&sys_key];
                let is_descendant = hierarchy_analysis.reachable()[index(row, col, hg_node_count)];
                bitset.set(idx, is_descendant);
            }
        }

        let mut sets_with_conditions_of_systems =
            vec![FixedBitSet::with_capacity(set_with_conditions_count); sys_count];
        for &(col, sys_key) in &hg_systems {
            let i = dg_system_idx_map[&sys_key];
            let bitset = &mut sets_with_conditions_of_systems[i];
            for (idx, &row) in hg_set_with_conditions_idxs
                .iter()
                .enumerate()
                .take_while(|&(_idx, &row)| row < col)
            {
                let is_ancestor = hierarchy_analysis.reachable()[index(row, col, hg_node_count)];
                bitset.set(idx, is_ancestor);
            }
        }

        SystemSchedule {
            systems: Vec::with_capacity(sys_count),
            system_conditions: Vec::with_capacity(sys_count),
            set_conditions: Vec::with_capacity(set_with_conditions_count),
            system_ids: dg_system_ids,
            set_ids: hg_set_ids,
            system_dependencies,
            system_dependents,
            sets_with_conditions_of_systems,
            systems_in_sets_with_conditions,
        }
    }
    // fn dot(&self) {
    //     println!(
    //         "{:?}",
    //         Dot::with_attr_getters(
    //             &self.hierarchy.graph,
    //             &[],
    //             &|g, edge| { String::new() },
    //             &|g, node| { format!(r#"label = "{}""#, self.get_node_name(&node.0)) }
    //         )
    //     );

    //     println!(
    //         "{:?}",
    //         Dot::with_attr_getters(
    //             &self.dependency.graph,
    //             &[],
    //             &|g, edge| { String::new() },
    //             &|g, node| { format!(r#"label = "{}""#, self.get_node_name(&node.0)) }
    //         )
    //     );
    // }
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
