use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::ops::Deref;

#[macro_use]
#[cfg(feature = "diesel")]
pub mod diesel;

#[cfg(feature = "itertools")]
pub mod itertools;

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

pub mod prelude {
    pub use super::{Entity, EntityBTreeMap, EntityHashMap};

    #[cfg(feature = "itertools")]
    pub use crate::itertools::prelude::*;
}

pub trait Entity {
    type Id: Copy + Eq + Hash;

    fn id(&self) -> Self::Id;
}

impl<E, F> Entity for F
where
    E: Entity,
    F: Deref<Target = E>,
{
    type Id = E::Id;

    fn id(&self) -> Self::Id {
        self.deref().id()
    }
}

pub type EntityBTreeMap<T> = BTreeMap<<T as Entity>::Id, T>;
pub type EntityHashMap<T> = HashMap<<T as Entity>::Id, T>;
