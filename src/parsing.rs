use core::ops;
use core::str::FromStr;

use nom::{InputIter, InputLength, InputTake, InputTakeAtPosition};

// TODO: Export a prelude?
pub use nom;
pub use nom::error::{FromExternalError, ParseError};
pub use nom::{AsChar, IResult, Parser, Finish};
pub use nom_supreme::ParserExt;

// TODO: It's unfortunate that `nom_derive` forces `InputIter<Item = u8>`, because it would be
//       neat to have derives, especially for structs with trivial parsers.
//       There are probably workarounds, or even solutions to upstream, but it is not currently
//       worth the extra time.
pub trait CharParse<I, E>: Sized
where
    I: CharInput,
    E: ParseError<I>,
{
    fn parse(input: I) -> IResult<I, Self, E>;

    fn pre_from_str(result: IResult<I, Self, E>) -> IResult<I, Self, E> {
        result
    }

    fn from_str_impl(s: I) -> Result<Self, E> {
        { |s| Self::pre_from_str(Self::parse(s)) }
            .complete()
            .all_consuming()
            .parse(s)
            .finish()
            .map(|(_, value)| value)
    }
}

#[macro_export]
macro_rules! CharParseFromStr {
    (@args () $vis:vis $name:ident $($token:tt)+) => {
        paste::paste! {
            CharParseFromStr! {@impl [<Parse $name Error>], $vis $name $($token)+}
        }
    };
    (@args (Err = $err:ty) $vis:vis $name:ident $($token:tt)+) => {
        paste::paste! {
            CharParseFromStr! {@impl err, $vis $name $($token)+}
        }
    };
    (@impl $err:ty, $vis:vis $name:ident $($token:tt)+) => {
        impl FromStr for $name {
            type Err = $err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <Self as CharParse::<_, _>>::from_str_impl(s)
            }
        }
    };
    (($($err:tt)*) $vis:vis $type_kind:ident $name:ident $($body:tt)+) => {
        CharParseFromStr! {@args ($($err)*) $vis $name $($body:tt)+}
    };
}

#[macro_export]
macro_rules! naive_parse_error {
    (@impl $name:ty, $msg:literal) => {
        naive_parse_error! {
            @impl $name
            [paste::paste! {
                #[derive(Debug, PartialEq)]
                #[derive(Error)]
                pub enum [<Parse $name Error>] {
                    #[error($msg)]
                    Invalid,
                }
            }]
        }
    };
    (@impl $name:ty [$($out:tt)*]) => {
        $($out)*

        ::paste::paste!{
            impl<I> ParseError<I> for [<Parse $name Error>] {
                fn from_error_kind(_input: I, _kind: nom::error::ErrorKind) -> Self {
                    Self::Invalid
                }
    
                fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
                    other
                }
            }
    
            impl<I, E> FromExternalError<I, E> for [<Parse $name Error>] {
                fn from_external_error(_input: I, _kind: nom::error::ErrorKind, _e: E) -> Self {
                    Self::Invalid
                }
            }
        }
    };
    ($name:ty, $msg:literal) => { naive_parse_error!(@impl $name, $msg); };
    ($name:ty) => { naive_parse_error! (@impl $name []); };
}

// TODO: This is silly. I want trait aliases. I want associated bounds. reeeee.

pub type FromStrErr<T> = <T as FromStr>::Err;
pub trait FromExternalFromStrErr<I, O>
where
    O: FromStr,
    Self: ParseError<I> + FromExternalError<I, FromStrErr<O>>,
{
}

impl<I, O, E> FromExternalFromStrErr<I, O> for E
where
    O: FromStr,
    E: ParseError<I> + FromExternalError<I, FromStrErr<O>>,
{
}

pub trait DoubleEndedInputIter
where
    Self: InputIter<IterElem = Self::ThisIterElem>,
    // TODO: I don't know why this bound is not implied already
    Self::ThisIterElem: Iterator<Item = <Self as InputIter>::Item>,
{
    type ThisIterElem: DoubleEndedIterator;
}

impl<I> DoubleEndedInputIter for I
where
    I: InputIter,
    <I as InputIter>::IterElem: DoubleEndedIterator,
{
    type ThisIterElem = <Self as InputIter>::IterElem;
}

pub trait CharInput
where
    Self: Clone,
    Self: InputIter<Item = Self::ThisInputIterItem>
        + InputLength
        + InputTake
        + InputTakeAtPosition<Item = Self::ThisInputTakeAtPositionItem>,
    Self: nom::Slice<ops::Range<usize>>
        + nom::Slice<ops::RangeTo<usize>>
        + nom::Slice<ops::RangeFrom<usize>>
        + nom::Slice<ops::RangeFull>,
{
    type ThisInputIterItem: AsChar;
    type ThisInputTakeAtPositionItem: AsChar;
}

impl<I> CharInput for I
where
    I: Clone,
    I: InputIter + InputLength + InputTake + InputTakeAtPosition,
    <I as InputIter>::Item: AsChar,
    <I as InputTakeAtPosition>::Item: AsChar,
    Self: nom::Slice<ops::Range<usize>>
        + nom::Slice<ops::RangeTo<usize>>
        + nom::Slice<ops::RangeFrom<usize>>
        + nom::Slice<ops::RangeFull>,
{
    type ThisInputIterItem = <Self as InputIter>::Item;
    type ThisInputTakeAtPositionItem = <Self as InputTakeAtPosition>::Item;
}
