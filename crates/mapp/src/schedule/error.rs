use std::fmt;

use snafu::prelude::*;

use super::{
    dag::{DagCrossDependencyError, DagOverlappingGroupError, DiGraphToposortError},
    node::{NodeId, TaskKey},
};

/// Category of errors encountered during [`Schedule::initialize`](crate::schedule::Schedule::initialize).
#[non_exhaustive]
#[derive(Snafu, Debug)]
pub enum ScheduleBuildError {
    /// Tried to topologically sort the hierarchy of task sets.
    #[snafu(display("Failed to topologically sort the hierarchy of task sets"))]
    HierarchySort {
        source: DiGraphToposortError<NodeId>,
    },
    /// Tried to topologically sort the dependency graph.
    #[snafu(display("Failed to topologically sort the dependency graph"))]
    DependencySort {
        source: DiGraphToposortError<NodeId>,
    },
    /// Tried to topologically sort the flattened dependency graph.
    #[snafu(display("Failed to topologically sort the flattened dependency graph"))]
    FlatDependencySort {
        source: DiGraphToposortError<TaskKey>,
    },
    /// Tried to order a task (set) relative to a task set it belongs to.
    #[snafu(display("`{:?}` and `{:?}` have both `in_set` and `before`-`after` relationships (these might be transitive). This combination is unsolvable as a task cannot run before or after a set it belongs to.", source.a, source.b))]
    CrossDependency {
        source: DagCrossDependencyError<NodeId>,
    },
    /// Tried to order task sets that share tasks.
    #[snafu(display("`{:?}` and `{:?}` have a `before`-`after` relationship (which may be transitive) but share tasks.", source.a_key, source.b_key))]
    SetsHaveOrderButIntersect{
        source: DagOverlappingGroupError<TaskSetKey>,
    },
    /// Tried to order a task (set) relative to all instances of some task function.
    TaskTypeSetAmbiguity{
        source: TaskTypeSetAmbiguityError,
    },
    /// A warning that was elevated to an error.
    Elevated{source: ScheduleBuildWarning},
}


/// Category of warnings encountered during [`Schedule::initialize`](crate::schedule::Schedule::initialize).
#[non_exhaustive]
#[derive(Snafu, Debug)]
pub enum ScheduleBuildWarning {
    /// The hierarchy of system sets contains redundant edges.
    ///
    /// This warning is **enabled** by default, but can be disabled by setting
    /// [`ScheduleBuildSettings::hierarchy_detection`] to [`LogLevel::Ignore`]
    /// or upgraded to a [`ScheduleBuildError`] by setting it to [`LogLevel::Error`].
    ///
    /// [`ScheduleBuildSettings::hierarchy_detection`]: crate::schedule::ScheduleBuildSettings::hierarchy_detection
    /// [`LogLevel::Ignore`]: crate::schedule::LogLevel::Ignore
    /// [`LogLevel::Error`]: crate::schedule::LogLevel::Error
    // #[snafu("The hierarchy of system sets contains redundant edges: {0:?}")]
    // HierarchyRedundancy(#[from] DagRedundancyError<NodeId>),
    /// Systems with conflicting access have indeterminate run order.
    ///
    /// This warning is **disabled** by default, but can be enabled by setting
    /// [`ScheduleBuildSettings::ambiguity_detection`] to [`LogLevel::Warn`]
    /// or upgraded to a [`ScheduleBuildError`] by setting it to [`LogLevel::Error`].
    ///
    /// [`ScheduleBuildSettings::ambiguity_detection`]: crate::schedule::ScheduleBuildSettings::ambiguity_detection
    /// [`LogLevel::Warn`]: crate::schedule::LogLevel::Warn
    /// [`LogLevel::Error`]: crate::schedule::LogLevel::Error
    // #[error(transparent)]
    // Ambiguity(#[from] AmbiguousSystemConflictsWarning),
}
