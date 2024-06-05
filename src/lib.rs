use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

#[macro_use]
#[cfg(feature = "diesel")]
pub mod diesel;

#[macro_use]
#[cfg(feature = "parsing")]
pub mod parsing;

#[cfg(feature = "udf-cell")]
pub mod udf_cell;

#[macro_use]
#[cfg(feature = "ulid-id")]
pub mod ulid_id;

#[cfg(feature = "unsigned")]
pub mod unsigned;

pub mod update;

pub trait Entity: Clone {
    type Id: Copy + Eq + Hash;

    fn id(&self) -> Self::Id;
}

pub type EntityBTreeMap<T> = BTreeMap<<T as Entity>::Id, T>;
pub type EntityHashMap<T> = HashMap<<T as Entity>::Id, T>;
