use core::marker::PhantomData;

pub use itertools;

pub mod prelude {
    pub use itertools::Itertools;

    pub use super::FulcrimItertools;
}

pub trait FulcrimItertools: Iterator {
    /// Convert each `Result::Ok` item of the iterator using the [`Into`] trait. `Result::Err`
    /// values are unchanged.
    ///
    /// Equivalent to `map_ok(Into::into)`, following the precedent of `map_into`.
    ///
    /// ```
    /// use fulcrim::itertools::FulcrimItertools;
    ///
    /// let input = vec![Ok(42i32), Err(false), Ok(12)];
    /// let it = input.into_iter().map_ok_into();
    /// itertools::assert_equal(it, vec![Ok(42f64), Err(false), Ok(12.)]);
    /// ```
    fn map_ok_into<T, U, E>(self) -> MapOkInto<Self, U>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        T: Into<U>,
    {
        map_ok_into(self)
    }

    /// Convert each `Result::Err` item of the iterator using the [`Into`] trait. `Result::Ok`
    /// values are unchanged.
    ///
    /// Useful for app/library error types that are composed from multiple sub-error types, for
    /// each of which it impls [`From`]. The `?` operator doesn't work "through" the iterator
    /// boundary.
    ///
    /// Follows `map_into` precedent in the orthogonal direction.
    ///
    /// ```
    /// use core::num::ParseIntError;
    ///
    /// use std::io;
    ///
    /// use fulcrim::itertools::FulcrimItertools;
    /// use thiserror::Error;
    ///
    /// pub fn process_cringe<'i>(input: impl Iterator<Item = &'i str>) -> Vec<Result<u32, AppError>> {
    ///     input.map(|s| s.parse().map_err(Into::into)).collect()
    /// }
    ///
    /// pub fn process<'i>(input: impl Iterator<Item = &'i str>) -> Vec<Result<u32, AppError>> {
    ///     input.map(str::parse).map_err_into().collect()
    /// }
    ///
    /// #[derive(Debug)]
    /// #[derive(Error)]
    /// pub enum AppError {
    ///     #[error(transparent)]
    ///     Io(#[from] io::Error),
    ///     #[error(transparent)]
    ///     ParseInt(#[from] ParseIntError),
    /// }
    /// ```
    fn map_err_into<T, E, E2>(self) -> MapErrInto<Self, E2>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        E: Into<E2>,
    {
        map_err_into(self)
    }

    /// Is to `map_ok` as [`Result::and_then`] is to [`Result::map`]
    ///
    /// Input iterator is of `Result<T, E>`, function is passed `T`, function returns `Result<U, E>`
    /// instead of just `U`, output iterator is of `Result<U, E>`
    ///
    /// Has a BUILT-IN, IMPLICIT `map_err_into` (because it's needed in MOST situations where this
    /// is useful), so the output `Result` can have a different error type as long as the input
    /// error type can be converted to it with [`From`].
    ///
    /// TODO: Hopefully there aren't too many cases where inference fails for `E2`...
    fn map_and_then<F, T, U, E, E2>(self, f: F) -> MapAndThen<Self, F>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        E: Into<E2>,
        F: FnMut(T) -> Result<U, E2>,
    {
        map_and_then(self, f)
    }

    /// Fallibly convert each item of the iterator using the [`FromInto`] trait.
    ///
    /// Included for completeness. I don't see this being nearly as useful as the clunkier-named
    /// ["and then try into"](`map_and_then_try_into`) variant.
    fn map_try_into<T, U>(self) -> MapTryInto<Self, T, U>
    where
        Self: Sized + Iterator<Item = T>,
        T: TryInto<U>,
    {
        map_try_into(self)
    }

    /// Fallibly convert each `Result::Err` item of the iterator using the [`Into`] trait.
    /// `Result::Ok` values are unchanged.
    ///
    /// Is to `map_and_then` as `map_ok_into` is to `map_ok`.
    ///
    /// TODO: This is where I KNOW that inference frequently fails for `E2`...
    fn map_and_then_try_into<T, U, E, E2>(self) -> MapAndThenTryInto<Self, T, U, E, E2>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        T: TryInto<U>,
        E2: From<E> + From<T::Error>,
    {
        map_and_then_try_into(self)
    }
}

impl<T> FulcrimItertools for T where T: Iterator {}

pub type MapOkInto<I, U> = MapSpecialCase<I, MapSpecialCaseFnOkInto<U>>;

#[derive(Clone, Debug)]
pub struct MapSpecialCaseFnOkInto<U>(PhantomData<U>);

impl<T, U, E> MapSpecialCaseFn<Result<T, E>> for MapSpecialCaseFnOkInto<U>
where
    T: Into<U>,
{
    type Out = Result<U, E>;

    fn call(&mut self, t: Result<T, E>) -> Self::Out {
        t.map(Into::into)
    }
}

pub fn map_ok_into<I, T, U, E>(iter: I) -> MapOkInto<I, U>
where
    I: IntoIterator<Item = Result<T, E>>,
    T: Into<U>,
{
    MapSpecialCase {
        iter,
        f: MapSpecialCaseFnOkInto(PhantomData),
    }
}

pub type MapErrInto<I, E2> = MapSpecialCase<I, MapSpecialCaseFnErrInto<E2>>;

#[derive(Clone, Debug)]
pub struct MapSpecialCaseFnErrInto<E2>(PhantomData<E2>);

impl<T, E, E2> MapSpecialCaseFn<Result<T, E>> for MapSpecialCaseFnErrInto<E2>
where
    E: Into<E2>,
{
    type Out = Result<T, E2>;

    fn call(&mut self, t: Result<T, E>) -> Self::Out {
        t.map_err(Into::into)
    }
}

pub fn map_err_into<I, T, E, E2>(iter: I) -> MapErrInto<I, E2>
where
    I: Sized + Iterator<Item = Result<T, E>>,
    E: Into<E2>,
{
    MapSpecialCase {
        iter,
        f: MapSpecialCaseFnErrInto(PhantomData),
    }
}

pub type MapAndThen<I, F> = MapSpecialCase<I, MapSpecialCaseFnAndThen<F>>;

pub struct MapSpecialCaseFnAndThen<F>(F);

impl<F, T, U, E, E2> MapSpecialCaseFn<Result<T, E>> for MapSpecialCaseFnAndThen<F>
where
    E: Into<E2>,
    F: FnMut(T) -> Result<U, E2>,
{
    type Out = Result<U, E2>;

    fn call(&mut self, t: Result<T, E>) -> Self::Out {
        t.map_err(Into::into).and_then(|v| self.0(v))
    }
}

pub fn map_and_then<I, F, T, U, E, E2>(iter: I, f: F) -> MapAndThen<I, F>
where
    I: Iterator<Item = Result<T, E>>,
    E: Into<E2>,
    F: FnMut(T) -> Result<U, E2>,
{
    MapSpecialCase {
        iter,
        f: MapSpecialCaseFnAndThen(f),
    }
}

pub type MapTryInto<I, T, U> = MapSpecialCase<I, MapSpecialCaseFnTryInto<T, U>>;

#[derive(Clone, Debug)]
pub struct MapSpecialCaseFnTryInto<T, U>(PhantomData<(T, U)>);

impl<T, U> MapSpecialCaseFn<T> for MapSpecialCaseFnTryInto<T, U>
where
    T: TryInto<U>,
{
    type Out = Result<U, T::Error>;

    fn call(&mut self, t: T) -> Self::Out {
        t.try_into()
    }
}

pub fn map_try_into<I, T, U>(iter: I) -> MapTryInto<I, T, U>
where
    I: Iterator<Item = T>,
    T: TryInto<U>,
{
    MapSpecialCase {
        iter,
        f: MapSpecialCaseFnTryInto(PhantomData),
    }
}

pub type MapAndThenTryInto<I, T, U, E, E2> =
    MapSpecialCase<I, MapSpecialCaseFnAndThenTryInto<T, U, E, E2>>;

#[derive(Clone, Debug)]
pub struct MapSpecialCaseFnAndThenTryInto<T, U, E, E2>(PhantomData<(T, U, E, E2)>);

impl<T, U, E, E2> MapSpecialCaseFn<Result<T, E>> for MapSpecialCaseFnAndThenTryInto<T, U, E, E2>
where
    T: TryInto<U>,
    E2: From<E> + From<T::Error>,
{
    type Out = Result<U, E2>;

    fn call(&mut self, t: Result<T, E>) -> Self::Out {
        t.map_err(Into::into).and_then(|v| Ok(v.try_into()?))
    }
}

pub fn map_and_then_try_into<I, T, U, E, E2>(iter: I) -> MapAndThenTryInto<I, T, U, E, E2>
where
    I: Iterator<Item = Result<T, E>>,
    T: TryInto<U>,
    E2: From<E> + From<T::Error>,
{
    MapSpecialCase {
        iter,
        f: MapSpecialCaseFnAndThenTryInto(PhantomData),
    }
}

// Internal helper structs copied from `itertools` crate
// TODO: Propose making `MapSpecialCase` part of `itertools` public API?
mod map_special_case {
    #[derive(Clone, Debug)]
    #[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
    pub struct MapSpecialCase<I, F> {
        pub iter: I,
        pub f: F,
    }

    pub trait MapSpecialCaseFn<T> {
        type Out;

        fn call(&mut self, t: T) -> Self::Out;
    }

    impl<I, R> Iterator for MapSpecialCase<I, R>
    where
        I: Iterator,
        R: MapSpecialCaseFn<I::Item>,
    {
        type Item = R::Out;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next().map(|i| self.f.call(i))
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }

        fn fold<Acc, Fold>(self, init: Acc, mut fold_f: Fold) -> Acc
        where
            Fold: FnMut(Acc, Self::Item) -> Acc,
        {
            let mut f = self.f;
            self.iter.fold(init, move |acc, v| fold_f(acc, f.call(v)))
        }

        fn collect<C>(self) -> C
        where
            C: FromIterator<Self::Item>,
        {
            let mut f = self.f;
            self.iter.map(move |v| f.call(v)).collect()
        }
    }

    impl<I, R> DoubleEndedIterator for MapSpecialCase<I, R>
    where
        I: DoubleEndedIterator,
        R: MapSpecialCaseFn<I::Item>,
    {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.iter.next_back().map(|i| self.f.call(i))
        }
    }

    impl<I, R> ExactSizeIterator for MapSpecialCase<I, R>
    where
        I: ExactSizeIterator,
        R: MapSpecialCaseFn<I::Item>,
    {
    }
}
use map_special_case::{MapSpecialCase, MapSpecialCaseFn};
