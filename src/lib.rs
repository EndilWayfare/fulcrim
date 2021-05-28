use std::collections::HashMap;
use std::hash::Hash;

#[macro_use]
#[cfg(feature = "ulid-id")]
pub mod ulid_id;

pub trait Entity: Clone {
    type Id: Copy + Eq + Hash;

    fn id(&self) -> Self::Id;
}

pub type EntityHashMap<T> = HashMap<<T as Entity>::Id, T>;
