use std::any::{Any, TypeId};

use variadics_please::all_tuples;

use crate::{
    context::ContextError,
    coroutine::{LocalInjectRunnable, new_runnable},
};

use super::{
    TypeIdMap,
    condition::BoxCondition,
    graph_info::{Dependency, DependencyKind, GraphInfo},
    set::{InternedTaskSet, IntoTaskSet, TaskSet},
    task::ScheduleTask,
};

pub trait Schedulable {
    type Metadata;
    type GroupMetadata;

    fn into_config(self) -> ScheduleConfig<Self>
    where
        Self: Sized;
}

impl Schedulable for ScheduleTask {
    type Metadata = GraphInfo;
    type GroupMetadata = Chain;

    fn into_config(self) -> ScheduleConfig<Self> {
        ScheduleConfig {
            node: self,
            metadata: GraphInfo {
                hierarchy: Vec::new(),
                ..Default::default()
            },
            conditions: Vec::new(),
        }
    }
}

impl Schedulable for InternedTaskSet {
    type Metadata = GraphInfo;
    type GroupMetadata = Chain;

    fn into_config(self) -> ScheduleConfig<Self> {
        assert!(
            self.task_type().is_none(),
            "configuring task type sets is not allowed"
        );

        ScheduleConfig {
            node: self,
            metadata: GraphInfo::default(),
            conditions: Vec::new(),
        }
    }
}

pub struct ScheduleConfig<T: Schedulable> {
    pub(crate) node: T,
    pub(crate) metadata: T::Metadata,
    pub(crate) conditions: Vec<BoxCondition>,
}

pub enum ScheduleConfigs<T: Schedulable> {
    ScheduleConfig(ScheduleConfig<T>),
    Configs {
        configs: Vec<ScheduleConfigs<T>>,
        collective_conditions: Vec<BoxCondition>,
        metadata: T::GroupMetadata,
    },
}

impl<T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>> ScheduleConfigs<T> {
    /// Adds a new boxed system set to the systems.
    pub fn in_set_inner(&mut self, set: InternedTaskSet) {
        match self {
            Self::ScheduleConfig(config) => {
                config.metadata.hierarchy.push(set);
            }
            Self::Configs { configs, .. } => {
                for config in configs {
                    config.in_set_inner(set);
                }
            }
        }
    }

    fn before_inner(&mut self, set: InternedTaskSet) {
        match self {
            Self::ScheduleConfig(config) => {
                config
                    .metadata
                    .dependencies
                    .push(Dependency::new(DependencyKind::Before, set));
            }
            Self::Configs { configs, .. } => {
                for config in configs {
                    config.before_inner(set);
                }
            }
        }
    }

    fn after_inner(&mut self, set: InternedTaskSet) {
        match self {
            Self::ScheduleConfig(config) => {
                config
                    .metadata
                    .dependencies
                    .push(Dependency::new(DependencyKind::After, set));
            }
            Self::Configs { configs, .. } => {
                for config in configs {
                    config.after_inner(set);
                }
            }
        }
    }

    // fn before_ignore_deferred_inner(&mut self, set: InternedSystemSet) {
    //     match self {
    //         Self::ScheduleConfig(config) => {
    //             config
    //                 .metadata
    //                 .dependencies
    //                 .push(Dependency::new(DependencyKind::Before, set).add_config(IgnoreDeferred));
    //         }
    //         Self::Configs { configs, .. } => {
    //             for config in configs {
    //                 config.before_ignore_deferred_inner(set.intern());
    //             }
    //         }
    //     }
    // }

    // fn after_ignore_deferred_inner(&mut self, set: InternedSystemSet) {
    //     match self {
    //         Self::ScheduleConfig(config) => {
    //             config
    //                 .metadata
    //                 .dependencies
    //                 .push(Dependency::new(DependencyKind::After, set).add_config(IgnoreDeferred));
    //         }
    //         Self::Configs { configs, .. } => {
    //             for config in configs {
    //                 config.after_ignore_deferred_inner(set.intern());
    //             }
    //         }
    //     }
    // }

    // fn distributive_run_if_inner<M>(&mut self, condition: impl Condition<M> + Clone) {
    //     match self {
    //         Self::ScheduleConfig(config) => {
    //             config.conditions.push(new_condition(condition));
    //         }
    //         Self::Configs { configs, .. } => {
    //             for config in configs {
    //                 config.distributive_run_if_inner(condition.clone());
    //             }
    //         }
    //     }
    // }

    // fn ambiguous_with_inner(&mut self, set: InternedSystemSet) {
    //     match self {
    //         Self::ScheduleConfig(config) => {
    //             ambiguous_with(&mut config.metadata, set);
    //         }
    //         Self::Configs { configs, .. } => {
    //             for config in configs {
    //                 config.ambiguous_with_inner(set);
    //             }
    //         }
    //     }
    // }

    // fn ambiguous_with_all_inner(&mut self) {
    //     match self {
    //         Self::ScheduleConfig(config) => {
    //             config.metadata.ambiguous_with = Ambiguity::IgnoreAll;
    //         }
    //         Self::Configs { configs, .. } => {
    //             for config in configs {
    //                 config.ambiguous_with_all_inner();
    //             }
    //         }
    //     }
    // }

    // /// Adds a new boxed run condition to the systems.
    // ///
    // /// This is useful if you have a run condition whose concrete type is unknown.
    // /// Prefer `run_if` for run conditions whose type is known at compile time.
    // pub fn run_if_dyn(&mut self, condition: BoxedCondition) {
    //     match self {
    //         Self::ScheduleConfig(config) => {
    //             config.conditions.push(condition);
    //         }
    //         Self::Configs {
    //             collective_conditions,
    //             ..
    //         } => {
    //             collective_conditions.push(condition);
    //         }
    //     }
    // }

    fn chain_inner(mut self) -> Self {
        match &mut self {
            Self::ScheduleConfig(_) => { /* no op */ }
            Self::Configs { metadata, .. } => {
                metadata.set_chained();
            }
        };
        self
    }

    // fn chain_ignore_deferred_inner(mut self) -> Self {
    //     match &mut self {
    //         Self::ScheduleConfig(_) => { /* no op */ }
    //         Self::Configs { metadata, .. } => {
    //             metadata.set_chained_with_config(IgnoreDeferred);
    //         }
    //     }
    //     self
    // }
}

#[diagnostic::on_unimplemented(
    message = "`{Self}` does not describe a valid task configuration",
    label = "invalid task configuration"
)]
pub trait IntoScheduleConfigs<T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>, Marker>:
    Sized
{
    /// Convert into a [`ScheduleConfigs`].
    fn into_configs(self) -> ScheduleConfigs<T>;

    /// Add these systems to the provided `set`.
    #[track_caller]
    fn in_set(self, set: impl TaskSet) -> ScheduleConfigs<T> {
        self.into_configs().in_set(set)
    }

    fn before<M>(self, set: impl IntoTaskSet<M>) -> ScheduleConfigs<T> {
        self.into_configs().before(set)
    }

    fn after<M>(self, set: impl IntoTaskSet<M>) -> ScheduleConfigs<T> {
        self.into_configs().after(set)
    }

    // fn before_ignore_deferred<M>(self, set: impl IntoSystemSet<M>) -> ScheduleConfigs<T> {
    //     self.into_configs().before_ignore_deferred(set)
    // }

    // fn after_ignore_deferred<M>(self, set: impl IntoSystemSet<M>) -> ScheduleConfigs<T> {
    //     self.into_configs().after_ignore_deferred(set)
    // }

    // fn distributive_run_if<M>(self, condition: impl Condition<M> + Clone) -> ScheduleConfigs<T> {
    //     self.into_configs().distributive_run_if(condition)
    // }

    // fn run_if<M>(self, condition: impl Condition<M>) -> ScheduleConfigs<T> {
    //     self.into_configs().run_if(condition)
    // }

    // fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> ScheduleConfigs<T> {
    //     self.into_configs().ambiguous_with(set)
    // }

    // fn ambiguous_with_all(self) -> ScheduleConfigs<T> {
    //     self.into_configs().ambiguous_with_all()
    // }

    fn chain(self) -> ScheduleConfigs<T> {
        self.into_configs().chain()
    }

    // fn chain_ignore_deferred(self) -> ScheduleConfigs<T> {
    //     self.into_configs().chain_ignore_deferred()
    // }
}

impl<T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>> IntoScheduleConfigs<T, ()>
    for ScheduleConfigs<T>
{
    fn into_configs(self) -> Self {
        self
    }

    #[track_caller]
    fn in_set(mut self, set: impl TaskSet) -> Self {
        assert!(
            set.task_type().is_none(),
            "adding arbitrary systems to a system type set is not allowed"
        );

        self.in_set_inner(set.intern());

        self
    }

    fn before<M>(mut self, set: impl IntoTaskSet<M>) -> Self {
        let set = set.into_task_set();
        self.before_inner(set.intern());
        self
    }

    fn after<M>(mut self, set: impl IntoTaskSet<M>) -> Self {
        let set = set.into_task_set();
        self.after_inner(set.intern());
        self
    }

    // fn before_ignore_deferred<M>(mut self, set: impl IntoSystemSet<M>) -> Self {
    //     let set = set.into_system_set();
    //     self.before_ignore_deferred_inner(set.intern());
    //     self
    // }

    // fn after_ignore_deferred<M>(mut self, set: impl IntoSystemSet<M>) -> Self {
    //     let set = set.into_system_set();
    //     self.after_ignore_deferred_inner(set.intern());
    //     self
    // }

    // fn distributive_run_if<M>(
    //     mut self,
    //     condition: impl Condition<M> + Clone,
    // ) -> ScheduleConfigs<T> {
    //     self.distributive_run_if_inner(condition);
    //     self
    // }

    // fn run_if<M>(mut self, condition: impl Condition<M>) -> ScheduleConfigs<T> {
    //     self.run_if_dyn(new_condition(condition));
    //     self
    // }

    // fn ambiguous_with<M>(mut self, set: impl IntoSystemSet<M>) -> Self {
    //     let set = set.into_system_set();
    //     self.ambiguous_with_inner(set.intern());
    //     self
    // }

    // fn ambiguous_with_all(mut self) -> Self {
    //     self.ambiguous_with_all_inner();
    //     self
    // }

    fn chain(self) -> Self {
        self.chain_inner()
    }

    // fn chain_ignore_deferred(self) -> Self {
    //     self.chain_ignore_deferred_inner()
    // }
}

// impl<T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>> ScheduleConfigs<T> {
//     /// Adds a new boxed system set to the systems.
//     pub fn in_set_inner(&mut self, set: InternedSystemSet) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 config.metadata.hierarchy.push(set);
//             }
//             Self::Configs { configs, .. } => {
//                 for config in configs {
//                     config.in_set_inner(set);
//                 }
//             }
//         }
//     }

//     fn before_inner(&mut self, set: InternedSystemSet) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 config
//                     .metadata
//                     .dependencies
//                     .push(Dependency::new(DependencyKind::Before, set));
//             }
//             Self::Configs { configs, .. } => {
//                 for config in configs {
//                     config.before_inner(set);
//                 }
//             }
//         }
//     }

//     fn after_inner(&mut self, set: InternedSystemSet) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 config
//                     .metadata
//                     .dependencies
//                     .push(Dependency::new(DependencyKind::After, set));
//             }
//             Self::Configs { configs, .. } => {
//                 for config in configs {
//                     config.after_inner(set);
//                 }
//             }
//         }
//     }

//     fn before_ignore_deferred_inner(&mut self, set: InternedSystemSet) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 config
//                     .metadata
//                     .dependencies
//                     .push(Dependency::new(DependencyKind::Before, set).add_config(IgnoreDeferred));
//             }
//             Self::Configs { configs, .. } => {
//                 for config in configs {
//                     config.before_ignore_deferred_inner(set.intern());
//                 }
//             }
//         }
//     }

//     fn after_ignore_deferred_inner(&mut self, set: InternedSystemSet) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 config
//                     .metadata
//                     .dependencies
//                     .push(Dependency::new(DependencyKind::After, set).add_config(IgnoreDeferred));
//             }
//             Self::Configs { configs, .. } => {
//                 for config in configs {
//                     config.after_ignore_deferred_inner(set.intern());
//                 }
//             }
//         }
//     }

//     fn distributive_run_if_inner<M>(&mut self, condition: impl Condition<M> + Clone) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 config.conditions.push(new_condition(condition));
//             }
//             Self::Configs { configs, .. } => {
//                 for config in configs {
//                     config.distributive_run_if_inner(condition.clone());
//                 }
//             }
//         }
//     }

//     fn ambiguous_with_inner(&mut self, set: InternedSystemSet) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 ambiguous_with(&mut config.metadata, set);
//             }
//             Self::Configs { configs, .. } => {
//                 for config in configs {
//                     config.ambiguous_with_inner(set);
//                 }
//             }
//         }
//     }

//     fn ambiguous_with_all_inner(&mut self) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 config.metadata.ambiguous_with = Ambiguity::IgnoreAll;
//             }
//             Self::Configs { configs, .. } => {
//                 for config in configs {
//                     config.ambiguous_with_all_inner();
//                 }
//             }
//         }
//     }

//     /// Adds a new boxed run condition to the systems.
//     ///
//     /// This is useful if you have a run condition whose concrete type is unknown.
//     /// Prefer `run_if` for run conditions whose type is known at compile time.
//     pub fn run_if_dyn(&mut self, condition: BoxedCondition) {
//         match self {
//             Self::ScheduleConfig(config) => {
//                 config.conditions.push(condition);
//             }
//             Self::Configs {
//                 collective_conditions,
//                 ..
//             } => {
//                 collective_conditions.push(condition);
//             }
//         }
//     }

// fn chain_inner(mut self) -> Self {
//     match &mut self {
//         Self::ScheduleConfig(_) => { /* no op */ }
//         Self::Configs { metadata, .. } => {
//             metadata.set_chained();
//         }
//     };
//     self
// }

//     fn chain_ignore_deferred_inner(mut self) -> Self {
//         match &mut self {
//             Self::ScheduleConfig(_) => { /* no op */ }
//             Self::Configs { metadata, .. } => {
//                 metadata.set_chained_with_config(IgnoreDeferred);
//             }
//         }
//         self
//     }
// }

#[derive(Default)]
pub enum Chain {
    /// Tasks are independent. Nodes are allowed to run in any order.
    #[default]
    Unchained,
    /// Tasks are chained. `before -> after` ordering constraints
    /// will be added between the successive elements.
    Chained(TypeIdMap<Box<dyn Any>>),
}

impl Chain {
    /// Specify that the tasks must be chained.
    pub fn set_chained(&mut self) {
        if matches!(self, Chain::Unchained) {
            *self = Self::Chained(Default::default());
        };
    }
    /// Specify that the tasks must be chained, and add the specified configuration for
    /// all dependencies created between these tasks.
    pub fn set_chained_with_config<T: 'static>(&mut self, config: T) {
        self.set_chained();
        if let Chain::Chained(config_map) = self {
            config_map.insert(TypeId::of::<T>(), Box::new(config));
        } else {
            unreachable!()
        };
    }
}

impl<F, Args> IntoScheduleConfigs<ScheduleTask, Args> for F
where
    F: LocalInjectRunnable<Args, ContextError> + 'static,
    Args: 'static,
{
    fn into_configs(self) -> ScheduleConfigs<ScheduleTask> {
        ScheduleConfigs::ScheduleConfig(ScheduleTask::into_config(ScheduleTask::new(new_runnable(
            self,
        ))))
    }
}

impl<S: TaskSet> IntoScheduleConfigs<InternedTaskSet, ()> for S {
    fn into_configs(self) -> ScheduleConfigs<InternedTaskSet> {
        ScheduleConfigs::ScheduleConfig(InternedTaskSet::into_config(self.intern()))
    }
}

macro_rules! impl_node_type_collection {
    ($(($param: ident, $task: ident)),*) => {
        impl<$($param, $task),*, T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>> IntoScheduleConfigs<T, ($($param,)*)> for ($($task,)*)
        where
            $($task: IntoScheduleConfigs<T, $param>),*
        {
            #[expect(
                clippy::allow_attributes,
                reason = "We are inside a macro, and as such, `non_snake_case` is not guaranteed to apply."
            )]
            #[allow(
                non_snake_case,
                reason = "Variable names are provided by the macro caller, not by us."
            )]
            fn into_configs(self) -> ScheduleConfigs<T> {
                let ($($task,)*) = self;
                ScheduleConfigs::Configs {
                    metadata: Default::default(),
                    configs: vec![$($task.into_configs(),)*],
                    collective_conditions: Vec::new(),
                }
            }
        }
    }
}

all_tuples!(impl_node_type_collection, 1, 20, P, S);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_configs() {
        {
            async fn test() -> Result<(), ContextError> {
                Ok(())
            }
            _ = test.into_configs();
        }

        async move || -> Result<(), ContextError> { Ok(()) }.into_configs();
    }
}
