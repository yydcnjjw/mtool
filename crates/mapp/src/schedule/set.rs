use std::hash::Hash;
use std::{any::TypeId, hash::Hasher, marker::PhantomData};
use std::{fmt, hash};

use crate::{define_label, intern::Interned};

pub use crate::label::DynEq;
pub use mapp_macros::{ScheduleLabel, TaskSet};

define_label!(
    /// A strongly-typed class of labels used to identify a [`Schedule`](crate::schedule::Schedule).
    #[diagnostic::on_unimplemented(
        note = "consider annotating `{Self}` with `#[derive(ScheduleLabel)]`"
    )]
    ScheduleLabel,
    SCHEDULE_LABEL_INTERNER
);

define_label!(
    /// Types that identify logical groups of systems.
    #[diagnostic::on_unimplemented(
        note = "consider annotating `{Self}` with `#[derive(TaskSet)]`"
    )]
    TaskSet,
    TASK_SET_INTERNER,
    extra_methods: {
        /// Returns `Some` if this system set is a [`TaskTypeSet`].
        fn task_type(&self) -> Option<TypeId> {
            None
        }

        /// Returns `true` if this system set is an [`AnonymousSet`].
        fn is_anonymous(&self) -> bool {
            false
        }
    },
    extra_methods_impl: {
        fn task_type(&self) -> Option<TypeId> {
            (**self).task_type()
        }

        fn is_anonymous(&self) -> bool {
            (**self).is_anonymous()
        }
    }
);

/// A shorthand for `Interned<dyn TaskSet>`.
pub type InternedTaskSet = Interned<dyn TaskSet>;
/// A shorthand for `Interned<dyn ScheduleLabel>`.
pub type InternedScheduleLabel = Interned<dyn ScheduleLabel>;

/// A [`TaskSet`] grouping instances of the same function.
///
/// This kind of set is automatically populated and thus has some special rules:
/// - You cannot manually add members.
/// - You cannot configure them.
/// - You cannot order something relative to one if it has more than one member.
pub struct TaskTypeSet<T: 'static>(PhantomData<fn() -> T>);

impl<T: 'static> TaskTypeSet<T> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> fmt::Debug for TaskTypeSet<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("TaskTypeSet")
            .field(&format_args!("fn {}()", &core::any::type_name::<T>()))
            .finish()
    }
}

impl<T> hash::Hash for TaskTypeSet<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        // all systems of a given type are the same
    }
}

impl<T> Clone for TaskTypeSet<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TaskTypeSet<T> {}

impl<T> PartialEq for TaskTypeSet<T> {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        // all systems of a given type are the same
        true
    }
}

impl<T> Eq for TaskTypeSet<T> {}

impl<T> TaskSet for TaskTypeSet<T> {
    fn task_type(&self) -> Option<TypeId> {
        Some(TypeId::of::<T>())
    }

    fn dyn_clone(&self) -> Box<dyn TaskSet> {
        Box::new(*self)
    }

    fn as_dyn_eq(&self) -> &dyn DynEq {
        self
    }

    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        TypeId::of::<Self>().hash(&mut state);
        self.hash(&mut state);
    }
}

/// A [`TaskSet`] implicitly created when using
/// [`Schedule::add_systems`](super::Schedule::add_systems) or
/// [`Schedule::configure_sets`](super::Schedule::configure_sets).
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct AnonymousSet(usize);

impl AnonymousSet {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

impl TaskSet for AnonymousSet {
    fn is_anonymous(&self) -> bool {
        true
    }

    fn dyn_clone(&self) -> Box<dyn TaskSet> {
        Box::new(*self)
    }

    fn as_dyn_eq(&self) -> &dyn DynEq {
        self
    }

    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        TypeId::of::<Self>().hash(&mut state);
        self.hash(&mut state);
    }
}

/// Types that can be converted into a [`TaskSet`].
///
/// # Usage notes
///
/// This trait should only be used as a bound for trait implementations or as an
/// argument to a function. If a system set needs to be returned from a function
/// or stored somewhere, use [`TaskSet`] instead of this trait.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a system set",
    label = "invalid system set"
)]
pub trait IntoTaskSet<Marker>: Sized {
    /// The type of [`TaskSet`] this instance converts into.
    type Set: TaskSet;

    /// Converts this instance to its associated [`TaskSet`] type.
    fn into_task_set(self) -> Self::Set;
}

// task sets
impl<S: TaskSet> IntoTaskSet<()> for S {
    type Set = Self;

    #[inline]
    fn into_task_set(self) -> Self::Set {
        self
    }
}

// // systems
// impl<Marker, F> IntoTaskSet<(IsFunctionTask, Marker)> for F
// where
//     Marker: 'static,
//     F: TaskParamFunction<Marker>,
// {
//     type Set = TaskTypeSet<FunctionTask<Marker, F>>;

//     #[inline]
//     fn into_system_set(self) -> Self::Set {
//         TaskTypeSet::<FunctionTask<Marker, F>>::new()
//     }
// }

// // exclusive systems
// impl<Marker, F> IntoTaskSet<(IsExclusiveFunctionTask, Marker)> for F
// where
//     Marker: 'static,
//     F: ExclusiveTaskParamFunction<Marker>,
// {
//     type Set = TaskTypeSet<ExclusiveFunctionTask<Marker, F>>;

//     #[inline]
//     fn into_system_set(self) -> Self::Set {
//         TaskTypeSet::<ExclusiveFunctionTask<Marker, F>>::new()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_schedule_label() {
        #[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct UnitLabel;

        #[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct TupleLabel(u32, u32);

        #[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct StructLabel {
            a: u32,
            b: u32,
        }

        #[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct EmptyTupleLabel();

        #[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct EmptyStructLabel {}

        #[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        enum EnumLabel {
            #[default]
            Unit,
            Tuple(u32, u32),
            Struct {
                a: u32,
                b: u32,
            },
        }

        #[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct GenericLabel<T>(PhantomData<T>);

        assert_eq!(UnitLabel.intern(), UnitLabel.intern());
        assert_eq!(EnumLabel::Unit.intern(), EnumLabel::Unit.intern());
        assert_ne!(UnitLabel.intern(), EnumLabel::Unit.intern());
        assert_ne!(UnitLabel.intern(), TupleLabel(0, 0).intern());
        assert_ne!(EnumLabel::Unit.intern(), EnumLabel::Tuple(0, 0).intern());

        assert_eq!(TupleLabel(0, 0).intern(), TupleLabel(0, 0).intern());
        assert_eq!(
            EnumLabel::Tuple(0, 0).intern(),
            EnumLabel::Tuple(0, 0).intern()
        );
        assert_ne!(TupleLabel(0, 0).intern(), TupleLabel(0, 1).intern());
        assert_ne!(
            EnumLabel::Tuple(0, 0).intern(),
            EnumLabel::Tuple(0, 1).intern()
        );
        assert_ne!(TupleLabel(0, 0).intern(), EnumLabel::Tuple(0, 0).intern());
        assert_ne!(
            TupleLabel(0, 0).intern(),
            StructLabel { a: 0, b: 0 }.intern()
        );
        assert_ne!(
            EnumLabel::Tuple(0, 0).intern(),
            EnumLabel::Struct { a: 0, b: 0 }.intern()
        );

        assert_eq!(
            StructLabel { a: 0, b: 0 }.intern(),
            StructLabel { a: 0, b: 0 }.intern()
        );
        assert_eq!(
            EnumLabel::Struct { a: 0, b: 0 }.intern(),
            EnumLabel::Struct { a: 0, b: 0 }.intern()
        );
        assert_ne!(
            StructLabel { a: 0, b: 0 }.intern(),
            StructLabel { a: 0, b: 1 }.intern()
        );
        assert_ne!(
            EnumLabel::Struct { a: 0, b: 0 }.intern(),
            EnumLabel::Struct { a: 0, b: 1 }.intern()
        );
        assert_ne!(
            StructLabel { a: 0, b: 0 }.intern(),
            EnumLabel::Struct { a: 0, b: 0 }.intern()
        );
        assert_ne!(
            StructLabel { a: 0, b: 0 }.intern(),
            EnumLabel::Struct { a: 0, b: 0 }.intern()
        );
        assert_ne!(StructLabel { a: 0, b: 0 }.intern(), UnitLabel.intern(),);
        assert_ne!(
            EnumLabel::Struct { a: 0, b: 0 }.intern(),
            EnumLabel::Unit.intern()
        );

        assert_eq!(
            GenericLabel::<u32>(PhantomData).intern(),
            GenericLabel::<u32>(PhantomData).intern()
        );
        assert_ne!(
            GenericLabel::<u32>(PhantomData).intern(),
            GenericLabel::<u64>(PhantomData).intern()
        );
    }

    #[test]
    fn test_derive_task_set() {
        #[derive(TaskSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct UnitSet;

        #[derive(TaskSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct TupleSet(u32, u32);

        #[derive(TaskSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct StructSet {
            a: u32,
            b: u32,
        }

        #[derive(TaskSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct EmptyTupleSet();

        #[derive(TaskSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct EmptyStructSet {}

        #[derive(TaskSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        enum EnumSet {
            #[default]
            Unit,
            Tuple(u32, u32),
            Struct {
                a: u32,
                b: u32,
            },
        }

        #[derive(TaskSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        struct GenericSet<T>(PhantomData<T>);

        assert_eq!(UnitSet.intern(), UnitSet.intern());
        assert_eq!(EnumSet::Unit.intern(), EnumSet::Unit.intern());
        assert_ne!(UnitSet.intern(), EnumSet::Unit.intern());
        assert_ne!(UnitSet.intern(), TupleSet(0, 0).intern());
        assert_ne!(EnumSet::Unit.intern(), EnumSet::Tuple(0, 0).intern());

        assert_eq!(TupleSet(0, 0).intern(), TupleSet(0, 0).intern());
        assert_eq!(EnumSet::Tuple(0, 0).intern(), EnumSet::Tuple(0, 0).intern());
        assert_ne!(TupleSet(0, 0).intern(), TupleSet(0, 1).intern());
        assert_ne!(EnumSet::Tuple(0, 0).intern(), EnumSet::Tuple(0, 1).intern());
        assert_ne!(TupleSet(0, 0).intern(), EnumSet::Tuple(0, 0).intern());
        assert_ne!(TupleSet(0, 0).intern(), StructSet { a: 0, b: 0 }.intern());
        assert_ne!(
            EnumSet::Tuple(0, 0).intern(),
            EnumSet::Struct { a: 0, b: 0 }.intern()
        );

        assert_eq!(
            StructSet { a: 0, b: 0 }.intern(),
            StructSet { a: 0, b: 0 }.intern()
        );
        assert_eq!(
            EnumSet::Struct { a: 0, b: 0 }.intern(),
            EnumSet::Struct { a: 0, b: 0 }.intern()
        );
        assert_ne!(
            StructSet { a: 0, b: 0 }.intern(),
            StructSet { a: 0, b: 1 }.intern()
        );
        assert_ne!(
            EnumSet::Struct { a: 0, b: 0 }.intern(),
            EnumSet::Struct { a: 0, b: 1 }.intern()
        );
        assert_ne!(
            StructSet { a: 0, b: 0 }.intern(),
            EnumSet::Struct { a: 0, b: 0 }.intern()
        );
        assert_ne!(
            StructSet { a: 0, b: 0 }.intern(),
            EnumSet::Struct { a: 0, b: 0 }.intern()
        );
        assert_ne!(StructSet { a: 0, b: 0 }.intern(), UnitSet.intern(),);
        assert_ne!(
            EnumSet::Struct { a: 0, b: 0 }.intern(),
            EnumSet::Unit.intern()
        );

        assert_eq!(
            GenericSet::<u32>(PhantomData).intern(),
            GenericSet::<u32>(PhantomData).intern()
        );
        assert_ne!(
            GenericSet::<u32>(PhantomData).intern(),
            GenericSet::<u64>(PhantomData).intern()
        );
    }
}
