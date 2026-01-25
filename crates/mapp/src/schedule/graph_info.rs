use std::any::{Any, TypeId};

use crate::intern::Interned;

use super::{
    TypeIdMap,
    set::{InternedTaskSet, ScheduleLabel, TaskSet},
};

/// Specifies what kind of edge should be added to the dependency graph.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub(crate) enum DependencyKind {
    /// A node that should be preceded.
    Before,
    /// A node that should be succeeded.
    After,
}

/// An edge to be added to the dependency graph.
pub(crate) struct Dependency {
    pub(crate) kind: DependencyKind,
    pub(crate) set: InternedTaskSet,
    pub(crate) options: TypeIdMap<Box<dyn Any>>,
}

impl Dependency {
    pub fn new(kind: DependencyKind, set: InternedTaskSet) -> Self {
        Self {
            kind,
            set,
            options: Default::default(),
        }
    }
    pub fn add_config<T: 'static>(mut self, option: T) -> Self {
        self.options.insert(TypeId::of::<T>(), Box::new(option));
        self
    }
}

/// Configures ambiguity detection for a single system.
#[derive(Clone, Debug, Default)]
pub(crate) enum Ambiguity {
    #[default]
    Check,
    /// Ignore warnings with systems in any of these system sets. May contain duplicates.
    IgnoreWithSet(Vec<InternedTaskSet>),
    /// Ignore all warnings.
    IgnoreAll,
}

/// Metadata about how the node fits in the schedule graph
#[derive(Default)]
pub struct GraphInfo {
    /// the sets that the node belongs to (hierarchy)
    pub(crate) hierarchy: Vec<InternedTaskSet>,
    /// the sets that the node depends on (must run before or after)
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) ambiguous_with: Ambiguity,
}
