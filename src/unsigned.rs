use core::fmt::{self, Display, Write};
use core::num::{
    NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8,
};
use core::str::FromStr;

use macro_attr_2018::macro_attr;
use nom::error::{FromExternalError, ParseError};
use num_integer::Integer;
use num_traits::{CheckedAdd, CheckedMul, One, ToPrimitive, Unsigned};
use thiserror::Error;

use crate::parsing::{AsChar, IResult, Parser, ParserExt};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::parsing::{CharInput, CharParse, DoubleEndedInputIter, FromExternalFromStrErr};

// TODO: Put `BijectiveK26` everywhere you inisted you were eventually going to

// NOTE: https://en.wikipedia.org/wiki/Bijective_numeration#The_bijective_base-26_system
macro_attr! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct BijectiveK26<T>(pub T) where T: Integer + Unsigned;
}

impl<T: Integer + Unsigned> BijectiveK26<T>
where
    T: From<u8> + CheckedAdd + CheckedMul + Clone,
{
    pub fn try_from_iter<I>(iter: I) -> Result<Self, ParseBijectiveK26Error>
    where
        I: Iterator<Item = char> + DoubleEndedIterator,
    {
        type Error = ParseBijectiveK26Error;

        let factors = itertools::iterate(T::one(), |i| i.clone() * T::from(26));
        iter.into_iter()
            .rev()
            .zip(factors)
            .map(|(c, factor)| {
                if c.is_ascii_alphabetic() {
                    let numeric_value = c.to_ascii_uppercase() as u8 - UPPERCASE_ASCII_OFFSET;
                    let term = factor
                        .checked_mul(&T::from(numeric_value))
                        .ok_or(Error::Overflow)?;

                    Ok(term)
                } else {
                    Err(Error::NonAsciiCharacter(c))
                }
            })
            .try_fold(T::zero(), |acc, item| {
                item.and_then(|addend| acc.checked_add(&addend).ok_or(Error::Overflow))
            })
            .and_then(|result| {
                if result != T::zero() {
                    Ok(result - T::one())
                } else {
                    Err(Error::EmptyString)
                }
            })
            .map(Self)
    }
}

impl<T: Integer + Unsigned> FromStr for BijectiveK26<T>
where
    T: From<u8> + CheckedAdd + CheckedMul + Clone,
{
    type Err = ParseBijectiveK26Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: We can statically determine the bijective form of `u32::MAX` and reject strings
        //       that are shortlex larger.
        Self::try_from_iter(s.chars())
    }
}

#[derive(Debug, PartialEq)]
#[derive(Error)]
pub enum ParseBijectiveK26Error {
    #[error("{:?} is not an ascii letter", 0)]
    NonAsciiCharacter(char),
    #[error("String was empty")]
    EmptyString,
    #[error("Value was too large")]
    Overflow,
}

impl<T: Integer + Unsigned, I, E> CharParse<I, E> for BijectiveK26<T>
where
    T: From<u8> + CheckedAdd + CheckedMul + Clone,
    I: CharInput,
    I: DoubleEndedInputIter,
    E: FromExternalFromStrErr<I, Self>,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        nom::character::complete::alpha1
            .map(|alphas: I| alphas.iter_elements().map(AsChar::as_char))
            .map_res(BijectiveK26::try_from_iter)
            .parse(input)
    }
}

fn bijective_len<T: ToPrimitive>(k: u8, n: T) -> u8
{
    let k = f64::from(k);
    let n = n.to_f64().unwrap();
    let power = (n + 1.) * (k - 1.);

    power.log(k).floor() as u8
}

fn geometric_sum(k: u32, n: i32) -> i32 {
    let k = k as f64;
    let dividend = k.powi(n + 1) - k;
    let divisor = k - 1.;

    (dividend / divisor).floor() as i32
}

const UPPERCASE_ASCII_OFFSET: u8 = b'A' - 1;

impl<T: Integer + Unsigned> Display for BijectiveK26<T>
where
    T: From<u8> + Clone + HasGreaterWidth,
    T::NextGreater: Clone + ToPrimitive,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = T::NextGreater::from(self.0.clone()) + T::NextGreater::one();

        let mut remainder = n.to_f64().unwrap();
        let len = bijective_len(26u8, n);
        let largest_exponent = len as i32 - 1;
        let mut power = 26_f64.powi(largest_exponent);
        let mut min_remainder = 1. + geometric_sum(26, largest_exponent - 1) as f64;

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

macro_attr! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[derive(CharParseFromStr!)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Ordinal<T>(pub T) where T: Integer + Unsigned + NonZeroAble;
}

impl<T: Integer + Unsigned + NonZeroAble> Ordinal<T> {
    pub fn from_domain(x: T::NonZero) -> Self {
        Self(T::from(x) - T::one())
    }
}

impl<T: Integer + Unsigned + NonZeroAble, I, E> CharParse<I, E> for Ordinal<T>
where
    T: HasGreaterWidth,
    T: TryFrom<<T as HasGreaterWidth>::NextGreater>,
    <T as HasGreaterWidth>::NextGreater: NonZeroAble + CharParse<I, E>,
    I: CharInput,
    E: ParseError<I>,
    E: FromExternalError<I, ParseOrdinalError>,
{
    fn parse(input: I) -> IResult<I, Self, E> {
        // TODO: This could be made more robustly generic, but I am short on time

        nom::character::complete::digit1
            .and_then(T::NextGreater::parse)
            .map_res(|n| {
                <T as HasGreaterWidth>::NextGreater::try_from(n)
                    .map_err(|_| ParseOrdinalError::Invalid)
                    .and_then(|n| T::try_from(n).map_err(|_| ParseOrdinalError::Invalid))
                    .and_then(|n| T::NonZero::try_from(n).map_err(|_| ParseOrdinalError::Invalid))
            })
            .map(Self::from_domain)
            .parse(input)
    }
}

naive_parse_error!(Ordinal, "Failed to parse Ordinal");

impl<T: Unsigned + NonZeroAble> Display for Ordinal<T>
where
    T: Clone + HasGreaterWidth,
    <T as HasGreaterWidth>::NextGreater: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (T::NextGreater::from(self.0.clone()) + T::NextGreater::one()).fmt(f)
    }
}

pub trait HasGreaterWidth: Integer {
    type NextGreater: From<Self> + Integer + One;
}

macro_rules! impl_has_greater_width {
    ($t:ty, $next_greater: ty) => {
        impl HasGreaterWidth for $t {
            type NextGreater = $next_greater;
        }
    };
}

impl_has_greater_width!(u8, u16);
impl_has_greater_width!(u16, u32);
impl_has_greater_width!(u32, u64);

impl_has_greater_width!(i8, i16);
impl_has_greater_width!(i16, i32);
impl_has_greater_width!(i32, i64);

// TODO: `feature(generic_nonzero)` is in FCP and may stabilize soon
pub trait NonZeroAble: Integer + From<Self::NonZero> {
    type NonZero: TryFrom<Self>;
}

macro_rules! impl_non_zero_able {
    ($t:ty, $non_zero: ty) => {
        impl NonZeroAble for $t {
            type NonZero = $non_zero;
        }
    };
}

impl_non_zero_able!(u8, NonZeroU8);
impl_non_zero_able!(u16, NonZeroU16);
impl_non_zero_able!(u32, NonZeroU32);
impl_non_zero_able!(u64, NonZeroU64);

impl_non_zero_able!(i8, NonZeroI8);
impl_non_zero_able!(i16, NonZeroI16);
impl_non_zero_able!(i32, NonZeroI32);
impl_non_zero_able!(i64, NonZeroI64);

// TODO: Active ingredients (more or less) copypasta'd directly from `nom::character::complete`.
//       I'm not a huge fan of that decision, necessarily...
//       Or, well... I've customized it quite a bit and reviewed the whole thing, so... idk,
//       just needs to be revisited to make sure that smart.
macro_rules! impl_char_parse_for_uint {
    ($t:ty) => {
        impl<I, E> CharParse<I, E> for $t
        where
            I: CharInput,
            E: ParseError<I>,
        {
            fn parse(input: I) -> IResult<I, $t, E> {
                let i = input;

                if i.input_len() == 0 {
                    return Err(nom::Err::Error(E::from_error_kind(
                        i,
                        nom::error::ErrorKind::Digit,
                    )));
                }

                let mut value: $t = 0;
                for (pos, c) in i.iter_indices() {
                    match c.as_char().to_digit(10) {
                        None => {
                            if pos == 0 {
                                return Err(nom::Err::Error(E::from_error_kind(
                                    i,
                                    nom::error::ErrorKind::Digit,
                                )));
                            } else {
                                return Ok((i.slice(pos..), value));
                            }
                        }
                        Some(d) => match value.checked_mul(10).and_then(|v| v.checked_add(d as $t))
                        {
                            None => {
                                return Err(nom::Err::Error(E::from_error_kind(
                                    i,
                                    nom::error::ErrorKind::Digit,
                                )))
                            }
                            Some(v) => value = v,
                        },
                    }
                }

                Ok((i.slice(i.input_len()..), value))
            }
        }
    };
}

impl_char_parse_for_uint!(u8);
impl_char_parse_for_uint!(u16);
impl_char_parse_for_uint!(u32);
impl_char_parse_for_uint!(u64);

#[cfg(test)]
mod test {
    use super::*;

    mod ordinal {
        use super::*;

        // TODO: Flesh out

        #[test]
        fn works_at_all_for_u8() {
            assert_eq!("1".parse(), Ok(Ordinal::<u8>(0)));
            assert_eq!(&format!("{}", Ordinal::<u8>(0)), "1");
        }

        #[test]
        fn works_at_all_for_u16() {
            assert_eq!("1".parse(), Ok(Ordinal::<u16>(0)));
            assert_eq!(&format!("{}", Ordinal::<u16>(0)), "1");
        }

        #[test]
        fn works_at_all_for_u32() {
            assert_eq!("1".parse(), Ok(Ordinal::<u32>(0)));
            assert_eq!(&format!("{}", Ordinal::<u32>(0)), "1");
        }
    }

    mod bijective_k26 {
        use super::*;

        // TODO: Flesh out

        #[test]
        fn works_at_all_for_u8() {
            assert_eq!("A".parse(), Ok(BijectiveK26::<u8>(0)));
            assert_eq!(&format!("{}", BijectiveK26::<u8>(0)), "A");
        }

        #[test]
        fn works_at_all_for_u16() {
            assert_eq!("A".parse(), Ok(BijectiveK26::<u16>(0)));
            assert_eq!(&format!("{}", BijectiveK26::<u16>(0)), "A");
        }

        #[test]
        fn accepts_a() {
            let upper = "A";
            let lower = "a";

            let expected = Ok(BijectiveK26::<u32>(0));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_a() {
            assert_eq!(&BijectiveK26::<u32>(0).to_string(), "A");
        }

        #[test]
        fn accepts_z() {
            let upper = "Z";
            let lower = "z";

            let expected = Ok(BijectiveK26::<u32>(25));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_z() {
            assert_eq!(&BijectiveK26::<u32>(25).to_string(), "Z");
        }

        #[test]
        fn accepts_aa() {
            let upper = "AA";
            let lower = "aa";

            let expected = Ok(BijectiveK26::<u32>(26));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_aa() {
            assert_eq!(&BijectiveK26::<u32>(26).to_string(), "AA");
        }

        #[test]
        fn accepts_ab() {
            let upper = "AB";
            let lower = "ab";

            let expected = Ok(BijectiveK26::<u32>(27));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_ab() {
            assert_eq!(&BijectiveK26::<u32>(27).to_string(), "AB");
        }

        #[test]
        fn accepts_az() {
            let upper = "AZ";
            let lower = "az";

            let expected = Ok(BijectiveK26::<u32>(51));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_az() {
            assert_eq!(&BijectiveK26::<u32>(51).to_string(), "AZ");
        }

        #[test]
        fn accepts_ba() {
            let upper = "BA";
            let lower = "ba";

            let expected = Ok(BijectiveK26::<u32>(52));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_ba() {
            assert_eq!(&BijectiveK26::<u32>(52).to_string(), "BA");
        }

        #[test]
        fn accepts_bz() {
            let upper = "BZ";
            let lower = "bz";

            let expected = Ok(BijectiveK26::<u32>(77));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_bz() {
            assert_eq!(&BijectiveK26::<u32>(77).to_string(), "BZ");
        }

        #[test]
        fn accepts_za() {
            let upper = "ZA";
            let lower = "za";

            let expected = Ok(BijectiveK26::<u32>(676));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_za() {
            assert_eq!(&BijectiveK26::<u32>(676).to_string(), "ZA");
        }

        #[test]
        fn accepts_zz() {
            let upper = "ZZ";
            let lower = "zz";

            let expected = Ok(BijectiveK26::<u32>(701));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_zz() {
            assert_eq!(&BijectiveK26::<u32>(701).to_string(), "ZZ");
        }

        #[test]
        fn accepts_aaa() {
            let upper = "AAA";
            let lower = "aaa";

            let expected = Ok(BijectiveK26::<u32>(702));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_aaa() {
            assert_eq!(&BijectiveK26::<u32>(702).to_string(), "AAA");
        }

        #[test]
        fn accepts_baa() {
            let upper = "BAA";
            let lower = "baa";

            let expected = Ok(BijectiveK26::<u32>(1378));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_baa() {
            assert_eq!(&BijectiveK26::<u32>(1378).to_string(), "BAA");
        }

        #[test]
        fn accepts_bzz() {
            let upper = "BZZ";
            let lower = "bzz";

            let expected = Ok(BijectiveK26::<u32>(2053));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_bzz() {
            assert_eq!(&BijectiveK26::<u32>(2053).to_string(), "BZZ");
        }

        #[test]
        fn accepts_zzz() {
            let upper = "ZZZ";
            let lower = "zzz";

            let expected = Ok(BijectiveK26::<u32>(18_277));
            assert_eq!(upper.parse(), expected);
            assert_eq!(lower.parse(), expected);
        }

        #[test]
        fn displays_zzz() {
            assert_eq!(&BijectiveK26::<u32>(18_277).to_string(), "ZZZ");
        }

        // TODO
    }
}
