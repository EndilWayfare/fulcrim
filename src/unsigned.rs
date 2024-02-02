use std::fmt::{self, Display};
use std::num::{NonZeroU32, ParseIntError};
use std::str::FromStr;

use macro_attr_2018::macro_attr;
use newtype_derive_2018::NewtypeFrom;

macro_attr! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[derive(NewtypeFrom!)]
    pub struct Ordinal(pub u32 /* TODO: Make generic over unsigned sizes */);
}

impl Ordinal {
    pub fn from_domain(x: NonZeroU32) -> Self {
        Self(u32::from(x) - 1)
    }
}

impl Display for Ordinal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0 as usize + 1).fmt(f)
    }
}

impl FromStr for Ordinal {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NonZeroU32::from_str(s).map(Self::from_domain)
    }
}