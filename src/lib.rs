use std::collections::{HashMap};
use std::hash::Hash;

use serde::{de::DeserializeOwned, Serialize};

#[macro_use]
mod ulid_id;

pub trait Entity: Clone {
    type Id: Copy + Eq + Hash + Serialize + DeserializeOwned;

    fn id(&self) -> Self::Id;
}

pub type EntityHashMap<T> = HashMap<<T as Entity>::Id, T>;
