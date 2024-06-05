use core::borrow::BorrowMut;

pub trait Update {
    type Message;

    fn update(&mut self, msg: Self::Message) -> bool;
}

impl<T> Update for &mut T
where
    T: Update,
{
    type Message = T::Message;

    fn update(&mut self, msg: Self::Message) -> bool {
        (**self).update(msg)
    }
}

impl<T> Update for Option<T>
where
    T: Update,
{
    type Message = T::Message;

    fn update(&mut self, msg: Self::Message) -> bool {
        self.as_mut()
            .map(|inner| inner.update(msg))
            .unwrap_or(false)
    }
}

// TODO: Should `Update` and `UpdateWithContext` be more formally related?
pub trait UpdateWithContext {
    type Message;
    type Context;
    fn update_with_context(&mut self, msg: Self::Message, ctx: &Self::Context) -> bool;
}

impl<T> UpdateWithContext for &mut T
where
    T: UpdateWithContext,
{
    type Message = T::Message;
    type Context = T::Context;

    fn update_with_context(&mut self, msg: Self::Message, ctx: &Self::Context) -> bool {
        (**self).update_with_context(msg, ctx)
    }
}

impl<T> UpdateWithContext for Option<T>
where
    T: UpdateWithContext,
{
    type Message = T::Message;
    type Context = T::Context;

    fn update_with_context(&mut self, msg: Self::Message, ctx: &Self::Context) -> bool {
        self.as_mut()
            .map(|inner| inner.update_with_context(msg, ctx))
            .unwrap_or(false)
    }
}

/// Blanket trait to provide a convenience method for assigning props in `changed` or updating values in `update`.
pub trait NeqAssign<NEW> {
    /// If `self` and `new` aren't equal, assigns `new` to `self` and returns true, otherwise returns false.
    ///
    /// Short for "Not equal assign".
    ///
    /// # Example
    /// ```
    /// # use fulcrim::NeqAssign;
    /// let mut foo = 1;
    ///
    /// assert_eq!(foo.neq_assign(42), true);
    /// assert_eq!(foo, 42);
    ///
    /// assert_eq!(foo.neq_assign(42), false);
    /// ```
    fn neq_assign(&mut self, new: NEW) -> bool;
}

impl<T: BorrowMut<U>, U: PartialEq> NeqAssign<U> for T {
    fn neq_assign(&mut self, new: U) -> bool {
        self.neq_assign_by(new, |x, y| x == y)
    }
}

/// Blanket trait to provide a convenience method for assigning props in `changed` or updating values in `update`.
///
/// Like `neq_assign`, but for cases where `self` doesn't impl `PartialEq` or a nonstandard equality comparison is needed.
///
/// Useful for `Result<T, E: !PartialEq>`.
pub trait NeqAssignBy<NEW> {
    /// ```
    /// # use yewtil::{NeqAssign, NeqAssignBy};
    /// ##[derive(Clone, Debug)]
    /// struct NonComparableError;
    ///
    /// fn eq_by_ok<T, E>(a: &Result<T, E>, b: &Result<T, E>) -> bool
    /// where
    ///     T: PartialEq,
    /// {
    ///     match (a, b) {
    ///         (Ok(_), Err(_))
    ///         | (Err(_), Ok(_))
    ///         | (Err(_), Err(_)) => false,
    ///         (Ok(a), Ok(b)) => a == b,
    ///     }
    /// }
    ///
    /// let mut foo: Result<u32, NonComparableError> = Ok(1);
    ///
    /// // Won't compile
    /// // assert_eq!(foo.neq_assign(Ok(42)), true)
    ///
    /// assert_eq!(foo.neq_assign_by(Ok(42), eq_by_ok), true);
    /// assert_eq!(foo.clone().unwrap(), 42);
    ///
    /// assert_eq!(foo.neq_assign_by(Err(NonComparableError), eq_by_ok), true);
    /// assert!(foo.is_err());
    ///
    /// // The tradeoff: all assignments of an `Err` value will count as updates, even if they are
    /// // "the same" for all practical intents and purposes.
    /// assert_eq!(foo.neq_assign_by(Err(NonComparableError), eq_by_ok), true);
    /// assert_eq!(foo.neq_assign_by(Err(NonComparableError), eq_by_ok), true);
    /// assert_eq!(foo.neq_assign_by(Err(NonComparableError), eq_by_ok), true);
    /// ```
    ///
    fn neq_assign_by<F>(&mut self, new: NEW, eq: F) -> bool
    where
        F: FnOnce(&NEW, &NEW) -> bool;
}

impl<T: BorrowMut<U>, U> NeqAssignBy<U> for T {
    fn neq_assign_by<F>(&mut self, new: U, eq: F) -> bool
    where
        F: FnOnce(&U, &U) -> bool,
    {
        if !eq(self.borrow(), &new) {
            *self.borrow_mut() = new;
            true
        } else {
            false
        }
    }
}
