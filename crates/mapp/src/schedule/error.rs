use snafu::prelude::*;

/// Category of errors encountered during schedule construction.
#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum ScheduleBuildError {
    /// A task set contains itself.
    #[snafu(display("Task set `{task_set}` contains itself."))]
    HierarchyLoop { task_set: String },
    /// The hierarchy of system sets contains a cycle.
    #[snafu(display("Task set hierarchy contains cycle(s).\n{node}"))]
    HierarchyCycle { node: String },
    // /// The hierarchy of system sets contains redundant edges.
    // ///
    // /// This error is disabled by default, but can be opted-in using [`ScheduleBuildSettings`].
    // #[snafu("System set hierarchy contains redundant edges.\n{0}")]
    // HierarchyRedundancy(String),
    // /// A system (set) has been told to run before itself.
    #[snafu(display("Task set `{task_set}` depends on itself."))]
    DependencyLoop { task_set: String },
    /// The dependency graph contains a cycle.
    #[snafu(display("Task dependencies contain cycle(s).\n{node}"))]
    DependencyCycle { node: String },
    // /// Tried to order a system (set) relative to a system set it belongs to.
    // #[snafu("`{0}` and `{1}` have both `in_set` and `before`-`after` relationships (these might be transitive). This combination is unsolvable as a system cannot run before or after a set it belongs to.")]
    // CrossDependency(String, String),
    // /// Tried to order system sets that share systems.
    // #[snafu("`{0}` and `{1}` have a `before`-`after` relationship (which may be transitive) but share systems.")]
    // SetsHaveOrderButIntersect(String, String),
    // /// Tried to order a system (set) relative to all instances of some system function.
    // #[snafu("Tried to order against `{0}` in a schedule that has more than one `{0}` instance. `{0}` is a `SystemTypeSet` and cannot be used for ordering if ambiguous. Use a different set without this restriction.")]
    // SystemTypeSetAmbiguity(String),
    // /// Systems with conflicting access have indeterminate run order.
    // ///
    // /// This error is disabled by default, but can be opted-in using [`ScheduleBuildSettings`].
    // #[snafu("Systems with conflicting access have indeterminate run order.\n{0}")]
    // Ambiguity(String),
    // /// Tried to run a schedule before all of its systems have been initialized.
    // #[snafu("Systems in schedule have not been initialized.")]
    // Uninitialized,
}
