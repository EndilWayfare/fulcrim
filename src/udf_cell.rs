//! Reference-counted invalidatable smart pointer
//!
//! For `yew`, and other single-threaded, "unidirectional data flow" trees whose nodes update their
//! children nodes when their input props change, if the calculation of *new state* does not
//! require access to the *old state*, you can achieve valid mutable input props by weakening the
//! "definitely equal or definitely not-equal" guarantee to "definitely equal or possibly
//! not-equal".
//!
//! For validity, you only NEED the "definitely equal" case, because that's the only case that
//! can be safely assumed to be up-to-date as-is and skipped. For optimization, you need the
//! "definitely not-equal" case to avoid unnecessary re-renders for content that actually turns
//! out to be up-to-date. As the "cost to clone" data for mutation increases (in the case of large,
//! shared state objects), the relative cost of "potentially unnecessary re-renders" becomes an
//! acceptable tradeoff against "very likely unnecessary clones".
//!
//! The problem with using plain `Rc` (or `Mrc` -> `Irc` to enforce prop immutability) is that you
//! HAVE to clone the contents of the `Rc` for the "master" value to borrow mutably. So if there
//! are checks that are performed on the `Rc` contents that determine whether a change is even
//! necessary (e.g. on idempotent operations), you either have to duplicate `if/let`s (and similar)
//! from an immutable borrow for a subsequent mutable borrow if changes are approved or you just
//! have to borrow mutably and commit to a full clone regardless. Not as bad as a full clone EVERY
//! TIME the component re-renders for ANY REASON, but still not great.
//!
//! Using `Rc` -> `Weak` suffers similar issues, but just invalidates the `Weak`s when borrowing
//! mutably. Not as bad as a full clone, but still likely triggers a lot of string clones in
//! non-trivial interfaces (especially in components that aren't AUTISTICALLY optimized against
//! cloning strings). It WILL trigger a full clone just like `Mrc` if you pass down an `Rc` clone
//! instead of a `Weak` somewhere (or otherwise have more than one strong reference), but at least
//! it won't break. Also, `Weak` doesn't impl `PartialEq`, so you will have to manually impl that
//! trait for ALL of your `Properties` structs that contain them.
//!
//! Plus, any solution that involves pairs of pointer types presents challenges for *nesting*.
//! If you have an `Mrc` of a struct that contains an `Mrc`, the "down through props" version
//! should technically be an `Irc` of a struct that contains an `Irc`, BUT that would require
//! maintaining two different structs (or making them parametric over smart-pointer, or...) AND
//! manually writing custom "to Irc" recursion (or maybe some kind of silly trait, or...). Just, a
//! lot of work and complication for something that is technically correct but in a very pedantic,
//! rarely important kind of way.
//!
//! Interiorly-mutable smart pointers with epoch counters provide these "definitely equal or
//! possibly not-equal" semantics. In the `yew`-type case with "re-render on change" semantics,
//! you really only need a two-state epoch. If the value behind the pointer is not modified, the
//! epoch is unchanged and all references are still valid. If the value behind the pointer is
//! modified, it MUST trigger a re-render; the epoch is flipped, the state-tree immediately
//! updates, all invalidated references update their epoch and re-render themselves and their
//! children, there can't be any remaining invalid references in the props tree under state
//! branches that don't descend from them, and now every reference is on the same epoch again.
//!
//! This only extends to references that STAY IN PROPS. The guarantee doesn't extend to
//! references that get stashed somewhere where they aren't directly subject to the "re-render on
//! change" flow. They WILL, however, always be valid. You can always read the current, up-to-date
//! value from the pointer. You just can't assume that "equal means unchanged", just like you can't
//! for any other object with interior-mutable semantics.
//!
//! To help maintain the "always immediately re-render on change" pattern and avoid unintentional
//! mutable borrows in downstream nodes, the pointer is created via a special "token" when created
//! and may only be mutated via a reference to that same token. The uniqueness of the token acts as
//! a replacement for the uniqueness of a mutable borrow. Unfortunately, this check must be done
//! dynamically at runtime, like that of its inner `RefCell`.
//!
//! But, by externalising the "are you the owner" determination, the "master" pointer and the
//! "props" pointer are THE SAME TYPE instead of a pair. As far as I can tell, I've minimized the
//! ammount of monomorphization as well, or at least I was mindful of that goal in the design.

// TODO: Consider a simple wrap-incrementing `u32`. Alignment-wise, this shouldn't take up any more
//       space than a `bool`. Then it suddenly becomes much more robust against "oops I wasn't part
//       of the UDF update" with a 1/u32::MAX chance of false-positive instead of 1/2.
//       I KNOW YOU THINK YOU'RE VERY CLEVER FOR HAVING WORKED OUT THAT A SIMPLE FLIP-FLOP IS ALL
//       THAT IS MATHEMATICALLY REQUIRED, but ffs just bench it and if there's minimal difference
//       then who tf cares?
// TODO: `impl<T> Update for UdfCell<T> where T: Update`?
// TODO: This doesn't impact the `yew` case, but a more robust solution should *actually
//       invalidate* out-of-date pointers. Maybe with a `reconnect` method that allows the caller
//       to explicitly acknowledge "yes, I know this is a modified value and I'm updating".
// TODO: A simpler way to explain the motivating problem, insufficient alternative solutions, and
//       the current proposal might be to reorganize around "there are actually several different
//       reasons that data/props would need to be cloned, and the problem is coming as close as
//       possible to getting rid of ALL of them". Copy-on-passing-in-props, Copy-on-update,
//       Copy-on-html, etc.

use core::fmt::{self, Debug};

use std::cell::{self, Ref, RefCell};
use std::rc::Rc;

use thiserror::Error;

#[derive(Default)]
struct _Token;

/// Creates `UdfCell`s and guards mutable access to the ones created by it.
///
/// Witout this token, a `UdfCell` is effectively read-only.
#[derive(Default)]
pub struct UdfToken(Rc<_Token>);

// TODO: Replace with negative impl when stabilized
// TODO: Doctest is currently only way to make inline "compile_fail" tests
/// ```compile_fail,E0599
/// let token = EpocRcToken::default();
/// token.clone()
/// ```
fn _token_does_not_impl_clone() {}

impl UdfToken {
    fn _private_clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Debug for UdfToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UdfToken({:p})", Rc::as_ptr(&self.0))
    }
}

impl PartialEq<Self> for UdfToken {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl UdfToken {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn make_cell<T>(&self, value: T) -> UdfCell<T> {
        let epoch = Default::default();
        let inner = Rc::new(RefCell::new(UdfCellInner { epoch, value }));
        let token = self._private_clone();

        UdfCell {
            inner,
            epoch,
            token,
        }
    }
}

#[derive(Debug)]
pub struct UdfCell<T> {
    inner: Rc<RefCell<UdfCellInner<T>>>,
    epoch: bool,
    token: UdfToken,
}

#[derive(Clone, Debug)]
struct UdfCellInner<T> {
    epoch: bool,
    value: T,
}

impl<T> UdfCellInner<T> {
    fn next_epoch(&self) -> bool {
        !self.epoch
    }

    fn increment_epoch(&mut self) {
        dbg!(self.epoch);
        self.epoch = self.next_epoch();
        dbg!(self.epoch);
    }
}

// NOTE: Trivial dispatch of immutable `RefCell` behavior
impl<T> UdfCell<T> {
    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow().unwrap()
    }

    // NOTE: Including for completeness, but I think it's extraneous. If you think you have to call
    //       this, you probably don't or you're probably doing something wrong.
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, cell::BorrowError> {
        self.inner
            .try_borrow()
            .map(|r| Ref::map(r, |inner| &inner.value))
    }
}

// NOTE: Dispatch of owned `RefCell` behavior
impl<T> UdfCell<T> {
    // NOTE: The `into_inner` semantics of `Rc` or `RefCell` are different, and it's a tricky
    //       question which is more appropriate here. I think the answer is "neither exactly".
    //
    //       The overall semantics of `UdfCell` are something like "A single strong `Rc` (owned by
    //       whoever-also-owns-the-token) with 0 or more slightly-stronger `Weak`s".
    //
    //       If the token-bearer calls `into_inner`, they're done sharing the updates they make to
    //      `T` and are "taking it private" to do something else with it as an owned value. If it
    //       were 1..n `Rc`..`Weak`, all the `Weak`s would be invalidated. Instead, in this
    //       situation, all the remaining `UdfCell`s (if any) are now sharing what is effectively
    //       an immutable `T` among themselves.
    //
    //       Essentially the same thing happens when one of the "non-token-bearing" cells calls
    //       `into_inner`. It's more or less `to_owned`.
    //
    //       This "clone-on-write" behavior sounds a lot more like `Rc::make_mut`, but the
    //       `get_mut`/`make_mut` naming doesn't fit here because the value is unwrapped. `get_mut`
    //       would be more like `Rc::into_inner`, so we should provide `try_into_inner` for the
    //       "if I have to clone, I'd rather bail" behavior.
    //
    //       So, the most appropriate choice seems to be "infallible copy-on-write".
    pub fn into_inner(mut self) -> T
    where
        T: Clone,
    {
        Rc::make_mut(&mut self.inner);

        self.try_into_inner().unwrap()
    }

    pub fn try_into_inner(self) -> Option<T> {
        // NOTE: Ok, the final implementation of this makes that wall of text look silly. "Of
        //       course you just call `Rc::into_inner` and then `RefCell::into_inner`." Although
        //       this seems somewhat obvious in retrospect, the wall effectively justifies why this
        //       seems to be the correct, least surprising behavior.
        Rc::into_inner(self.inner).map(|inner| inner.into_inner().value)
    }
}

// NOTE: The core special sauce
struct Authorized<'a, T>(&'a mut UdfCell<T>);

impl<'a, T> Authorized<'a, T> {
    fn increment_epoch(&mut self) {
        self.0.inner.borrow_mut().increment_epoch();
        self.0.epoch = self.0.inner.borrow().epoch;
    }

    pub fn try_with_mut<F>(&mut self, f: F) -> Result<bool, cell::BorrowMutError>
    where
        F: FnOnce(&mut T) -> bool,
    {
        let changed = self
            .0
            .inner
            .try_borrow_mut()
            .map(|mut inner| f(&mut inner.value))?;

        if changed {
            self.0.inner.borrow_mut().increment_epoch();
            self.0.epoch = self.0.inner.borrow().epoch;
        }

        Ok(changed)
    }

    pub fn replace(&mut self, value: T) -> T {
        let epoch = self.0.inner.borrow().next_epoch();
        self.0.epoch = epoch;

        self.0.inner.replace(UdfCellInner { epoch, value }).value
    }

    pub fn replace_with<F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut T) -> T,
    {
        let epoch = self.0.inner.borrow().next_epoch();
        self.0.epoch = epoch;

        self.0
            .inner
            .replace_with(|inner| UdfCellInner {
                epoch,
                value: f(&mut inner.value),
            })
            .value
    }

    pub fn swap(&mut self, other: &mut Self) {
        self.increment_epoch();
        other.increment_epoch();

        self.0.inner.swap(&other.0.inner)
    }
}

// NOTE: Dispatch of mutable `RefCell` behavior, augmented with token requirement
impl<T> UdfCell<T> {
    fn authorize(&mut self, token: &UdfToken) -> Result<Authorized<'_, T>, BorrowMutError> {
        if self.token == *token {
            Ok(Authorized(self))
        } else {
            Err(TokenError.into())
        }
    }

    pub fn with_mut<F>(&mut self, token: &UdfToken, f: F) -> bool
    where
        F: FnOnce(&mut T) -> bool,
    {
        self.try_with_mut(token, f).unwrap()
    }

    pub fn try_with_mut<F>(&mut self, token: &UdfToken, f: F) -> Result<bool, BorrowMutError>
    where
        F: FnOnce(&mut T) -> bool,
    {
        self.authorize(token)
            .and_then(|mut auth| auth.try_with_mut(f).map_err(Into::into))
    }

    pub fn replace(&mut self, token: &UdfToken, value: T) -> T {
        self.authorize(token).unwrap().replace(value)
    }

    pub fn replace_with<F>(&mut self, token: &UdfToken, f: F) -> T
    where
        F: FnOnce(&mut T) -> T,
    {
        self.authorize(token).unwrap().replace_with(f)
    }

    pub fn swap(&mut self, token: &UdfToken, other: &mut Self, other_token: &UdfToken) {
        self.authorize(token)
            .unwrap()
            .swap(&mut other.authorize(other_token).unwrap())
    }
}

#[derive(Debug)]
#[derive(Error)]
#[error("Invalid token")]
pub struct TokenError;

#[derive(Debug)]
#[derive(Error)]
pub enum BorrowMutError {
    #[error(transparent)]
    Token(#[from] TokenError),
    #[error(transparent)]
    BorrowMut(#[from] cell::BorrowMutError),
}

// NOTE: Dispatch of mutable `RefCell<T> where T: Default`, augmented with token requirement
impl<T: Default> UdfCell<T> {
    pub fn take(&mut self, token: &UdfToken) -> T {
        self.replace(token, Default::default())
    }
}

impl<T> UdfCell<T> {
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T: PartialEq> PartialEq<Self> for UdfCell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token && self.ptr_eq(other) && self.epoch == other.epoch
    }
}

impl<T> Clone for UdfCell<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            epoch: self.epoch,
            token: self.token._private_clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokens_are_unique() {
        let a = UdfToken::new();
        let b = UdfToken::new();

        assert!(!Rc::ptr_eq(&a.0, &b.0));
        assert_ne!(a, b);
    }

    #[test]
    fn cloned_tokens_are_equal() {
        let a = UdfToken::new();
        let b = UdfToken(a.0.clone());

        assert!(Rc::ptr_eq(&a.0, &b.0));
        assert_eq!(a, b);
    }

    // TODO: Verify that unnecessary clones are not performed. Perhaps with `UdfToken<Mock>`, where
    //       `Mock` has a `Clone` implementation that tracks how many times it is cloned?

    #[test]
    fn cloned_cell_can_borrow() {
        let token = UdfToken::new();
        let master = token.make_cell("wow".to_string());
        let master_borrow = master.borrow();

        let clone = master.clone();
        let clone_borrow = clone.borrow();

        assert_eq!(*master_borrow, *clone_borrow);
    }

    #[test]
    fn invalidated_cells_can_see_new_value() {
        let token = UdfToken::new();
        let mut master = token.make_cell("wow".to_string());

        let clone = master.clone();
        assert_eq!(clone, master);
        assert_eq!(*clone.borrow(), *master.borrow());

        master.with_mut(&token, |s| {
            s.push_str(" so cool");
            true
        });
        assert_ne!(clone, master);
        assert_eq!(*clone.borrow(), *master.borrow());
        assert_eq!(&*clone.borrow(), "wow so cool");
    }

    #[test]
    fn with_mut_does_not_invalidate_if_unchanged() {
        let token = UdfToken::new();
        let mut master = token.make_cell("wow".to_string());

        let clone = master.clone();
        assert_eq!(clone, master);
        assert_eq!(*clone.borrow(), *master.borrow());

        master.with_mut(&token, |s| {
            if s == "woo" {
                s.push_str("hoo");
                true
            } else {
                false
            }
        });
        assert_eq!(clone, master);
        assert_eq!(*clone.borrow(), *master.borrow());
        assert_eq!(&*clone.borrow(), "wow");
    }

    #[test]
    fn into_inner_on_master_does_not_invalidate() {
        let token = UdfToken::new();
        let master = token.make_cell("wow".to_string());

        let clone = master.clone();
        let mut unwrapped = master.into_inner();

        assert_eq!(*clone.borrow(), *unwrapped);

        unwrapped.push_str(" so cool");
        assert_eq!(&unwrapped, "wow so cool");
        assert_eq!(&*clone.borrow(), "wow");
    }

    #[test]
    fn into_inner_on_clone_just_disconnects() {
        let token = UdfToken::new();
        let mut master = token.make_cell("wow".to_string());

        let clone = master.clone();
        let mut unwrapped = clone.into_inner();

        master.with_mut(&token, |s| {
            s.push_str(" so cool");
            true
        });
        unwrapped.push_str(" amazing");

        assert_eq!(&*master.borrow(), "wow so cool");
        assert_eq!(&unwrapped, "wow amazing");
    }
}
