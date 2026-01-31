mod condition;
mod config;
mod graph_info;
mod set;
mod schedule_graph;
mod scheduler;
mod task;

use crate::hash::NoOpHash;
use std::{any::TypeId, collections::HashMap};

pub(crate) type TypeIdMap<V> = HashMap<TypeId, V, NoOpHash>;

pub use self::set::*;
