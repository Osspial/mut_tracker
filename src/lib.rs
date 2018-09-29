use std::ops::{Deref, DerefMut};
use std::cell::Cell;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct MutTracker<T, K> {
    value: T,
    mutated: Cell<bool>,
    _key: PhantomData<K>
}

impl<T, K> MutTracker<T, K> {
    #[inline(always)]
    pub fn new(value: T) -> MutTracker<T, K> {
        MutTracker {
            value,
            mutated: Cell::new(true),
            _key: PhantomData
        }
    }

    #[inline(always)]
    pub fn was_mutated(this: &Self) -> bool {
        this.mutated.get()
    }

    /// Reset the "was mutated" flag to `false`.
    #[inline(always)]
    pub fn reset_was_mutated(this: &Self, _key: K) {
        this.mutated.set(false)
    }

    #[inline(always)]
    pub fn change_key<NK>(this: Self) -> MutTracker<T, NK> {
        MutTracker {
            value: this.value,
            mutated: this.mutated,
            _key: PhantomData
        }
    }
}

impl<T, K> Deref for MutTracker<T, K> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T, K> DerefMut for MutTracker<T, K> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        self.mutated.set(true);
        &mut self.value
    }
}

impl<T: Clone, K> Clone for MutTracker<T, K> {
    #[inline(always)]
    fn clone(&self) -> MutTracker<T, K> {
        MutTracker::new(self.value.clone())
    }

    #[inline(always)]
    fn clone_from(&mut self, source: &Self) {
        self.mutated.set(true);
        self.value.clone_from(source);
    }
}

impl<T, K> From<T> for MutTracker<T, K> {
    #[inline(always)]
    fn from(t: T) -> MutTracker<T, K> {
        MutTracker::new(t)
    }
}
