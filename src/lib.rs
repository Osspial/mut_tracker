use std::ops::{Deref, DerefMut};
use std::cell::Cell;

#[derive(Debug)]
pub struct MutTracker<T> {
    mutated: Cell<bool>,
    value: T
}

impl<T> MutTracker<T> {
    #[inline(always)]
    pub fn new(value: T) -> MutTracker<T> {
        MutTracker {
            mutated: Cell::new(true),
            value
        }
    }

    #[inline(always)]
    pub fn was_mutated(this: &Self) -> bool {
        this.mutated.get()
    }

    /// Reset the "was mutated" flag to `false`.
    ///
    /// This is unsafe because unsafe code is allowed to depend on this value being handled correctly.
    #[inline(always)]
    pub unsafe fn reset_was_mutated(this: &Self) {
        this.mutated.set(false)
    }
}

impl<T> Deref for MutTracker<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for MutTracker<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        self.mutated.set(true);
        &mut self.value
    }
}

impl<T: Clone> Clone for MutTracker<T> {
    #[inline(always)]
    fn clone(&self) -> MutTracker<T> {
        MutTracker::new(self.value.clone())
    }

    #[inline(always)]
    fn clone_from(&mut self, source: &Self) {
        self.mutated.set(true);
        self.value.clone_from(source);
    }
}

impl<T> From<T> for MutTracker<T> {
    #[inline(always)]
    fn from(t: T) -> MutTracker<T> {
        MutTracker::new(t)
    }
}
