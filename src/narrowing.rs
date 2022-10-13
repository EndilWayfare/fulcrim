use std::fmt::{self, Display};
use std::num::{NonZeroU32, ParseIntError};
use std::str::FromStr;

use macro_attr_2018::macro_attr;
use newtype_derive_2018::NewtypeFrom;

macro_attr! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[derive(NewtypeFrom!)]
    pub struct CountFromOne(pub u32 /* TODO: Make generic over unsigned sizes */);
}

impl CountFromOne {
    pub fn from_domain(x: NonZeroU32) -> Self {
        Self(u32::from(x) - 1)
    }
}

impl Display for CountFromOne {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

impl FromStr for CountFromOne {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NonZeroU32::from_str(s).map(Self::from_domain)
    }
}
