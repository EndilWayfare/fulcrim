use core::fmt::{self, Display, Write};
use core::num::{NonZeroU32, ParseIntError};
use core::ops;
use core::str::FromStr;

use itertools::Itertools;
use macro_attr_2018::macro_attr;
use newtype_derive_2018::NewtypeFrom;
use thiserror::Error;

// TODO: Put `BijectiveK26` everywhere you inisted you were eventually going to

// NOTE: https://en.wikipedia.org/wiki/Bijective_numeration#The_bijective_base-26_system
macro_attr! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[derive(NewtypeFrom!)]
    pub struct BijectiveK26(pub u32 /* TODO: Make generic over unsigned sizes */);
}

fn bijective_len(k: u32, n: u64) -> u32 {
    let k = k as f64;
    let n = n as f64;
    let power = (n + 1.) * (k - 1.);

    power.log(k).floor() as u32
}

fn geometric_sum(k: u32, n: i32) -> i32 {
    let k = k as f64;
    let dividend = k.powi(n + 1) - k;
    let divisor = k - 1.;

    (dividend / divisor).floor() as i32
}

const UPPERCASE_ASCII_OFFSET: u8 = b'A' - 1;

impl Display for BijectiveK26 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.0 as u64 + 1;

        let len = bijective_len(26, n);
        let largest_exponent = len as i32 - 1;
        let mut power = 26_f64.powi(largest_exponent);
        let mut min_remainder = 1. + geometric_sum(26, largest_exponent - 1) as f64;
        let mut remainder = n as f64;

        for _ in 0..len {
            let digit = (remainder - min_remainder) / power;
            remainder -= digit.floor() * power;
            power /= 26.;
            min_remainder -= power;

            let code = UPPERCASE_ASCII_OFFSET + digit as u8;
            f.write_char(code as char)?
        }

        Ok(())
    }
}

impl FromStr for BijectiveK26 {
    type Err = ParseBijectiveK26Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(ParseBijectiveK26Error::EmptyString)
        }
        // TODO: We can statically determine the bijective form of `u32::MAX` and reject strings
        //       that are shortlex larger.
        else {
            let factors = itertools::iterate(1, |i| i * 26);
            s.chars()
                .rev()
                .zip(factors)
                .map(|(c, factor)| {
                    if c.is_ascii_alphabetic() {
                        let numeric_value = c.to_ascii_uppercase() as u8 - UPPERCASE_ASCII_OFFSET;

                        Ok(factor * numeric_value as u32)
                    } else {
                        Err(ParseBijectiveK26Error::NonAsciiCharacter(c))
                    }
                })
                .fold_ok(0, ops::Add::add)
                .map(|result| result - 1)
                .map(Self)
        }
    }
}

#[derive(Debug, PartialEq)]
#[derive(Error)]
pub enum ParseBijectiveK26Error {
    #[error("{:?} is not an ascii letter", 0)]
    NonAsciiCharacter(char),
    #[error("String was empty")]
    EmptyString,
}

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
        (self.0 as u64 + 1).fmt(f)
    }
}

impl FromStr for Ordinal {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str(s).and_then(|n| {
            u32::try_from(n)
                .map(Self)
                .or_else(|_| NonZeroU32::from_str(s).map(Self::from_domain))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod count_from_one {
        // use super::*;

        // TODO:
    }

    mod alphabetic_u32 {
        use super::*;

        #[test]
        fn accepts_a() {
            let upper = "A";
            let lower = "a";

            let expected = Ok(BijectiveK26(0));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_a() {
            assert_eq!(&BijectiveK26(0).to_string(), "A");
        }

        #[test]
        fn accepts_z() {
            let upper = "Z";
            let lower = "z";

            let expected = Ok(BijectiveK26(25));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_z() {
            assert_eq!(&BijectiveK26(25).to_string(), "Z");
        }

        #[test]
        fn accepts_aa() {
            let upper = "AA";
            let lower = "aa";

            let expected = Ok(BijectiveK26(26));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_aa() {
            assert_eq!(&BijectiveK26(26).to_string(), "AA");
        }

        #[test]
        fn accepts_ab() {
            let upper = "AB";
            let lower = "ab";

            let expected = Ok(BijectiveK26(27));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_ab() {
            assert_eq!(&BijectiveK26(27).to_string(), "AB");
        }

        #[test]
        fn accepts_az() {
            let upper = "AZ";
            let lower = "az";

            let expected = Ok(BijectiveK26(51));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_az() {
            assert_eq!(&BijectiveK26(51).to_string(), "AZ");
        }

        #[test]
        fn accepts_ba() {
            let upper = "BA";
            let lower = "ba";

            let expected = Ok(BijectiveK26(52));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_ba() {
            assert_eq!(&BijectiveK26(52).to_string(), "BA");
        }

        #[test]
        fn accepts_bz() {
            let upper = "BZ";
            let lower = "bz";

            let expected = Ok(BijectiveK26(77));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_bz() {
            assert_eq!(&BijectiveK26(77).to_string(), "BZ");
        }

        #[test]
        fn accepts_za() {
            let upper = "ZA";
            let lower = "za";

            let expected = Ok(BijectiveK26(676));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_za() {
            assert_eq!(&BijectiveK26(676).to_string(), "ZA");
        }

        #[test]
        fn accepts_zz() {
            let upper = "ZZ";
            let lower = "zz";

            let expected = Ok(BijectiveK26(701));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_zz() {
            assert_eq!(&BijectiveK26(701).to_string(), "ZZ");
        }

        #[test]
        fn accepts_aaa() {
            let upper = "AAA";
            let lower = "aaa";

            let expected = Ok(BijectiveK26(702));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_aaa() {
            assert_eq!(&BijectiveK26(702).to_string(), "AAA");
        }

        #[test]
        fn accepts_baa() {
            let upper = "BAA";
            let lower = "baa";

            let expected = Ok(BijectiveK26(1378));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_baa() {
            assert_eq!(&BijectiveK26(1378).to_string(), "BAA");
        }

        #[test]
        fn accepts_bzz() {
            let upper = "BZZ";
            let lower = "bzz";

            let expected = Ok(BijectiveK26(2053));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_bzz() {
            assert_eq!(&BijectiveK26(2053).to_string(), "BZZ");
        }

        #[test]
        fn accepts_zzz() {
            let upper = "ZZZ";
            let lower = "zzz";

            let expected = Ok(BijectiveK26(18_277));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_zzz() {
            assert_eq!(&BijectiveK26(18_277).to_string(), "ZZZ");
        }

        // TODO
    }
}
