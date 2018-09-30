use std::ptr::NonNull;
use std::ops::{Deref, DerefMut};
use std::cell::Cell;
use std::marker::PhantomData;
use std::fmt;


pub struct MoveMutSentinel<K> {
    self_ptr: Cell<NonNull<MoveMutSentinel<K>>>,
    _key: PhantomData<K>
}

pub struct MoveMutTracker<T, K> {
    value: T,
    sentinel: MoveMutSentinel<K>
}

impl<T, K> MoveMutTracker<T, K> {
    #[inline(always)]
    pub fn new(value: T) -> MoveMutTracker<T, K> {
        MoveMutTracker {
            value,
            sentinel: MoveMutSentinel::mutated()
        }
    }

    #[inline(always)]
    pub fn was_moved_or_mutated(this: &Self) -> bool {
        this.sentinel.was_moved_or_mutated()
    }

    #[inline(always)]
    pub fn set_unmutated(this: &Self, _key: K) {
        this.sentinel.set_unmutated(_key);
    }

    #[inline(always)]
    pub fn change_key<NK>(this: Self) -> MoveMutTracker<T, NK> {
        MoveMutTracker {
            value: this.value,
            sentinel: this.sentinel.change_key()
        }
    }
}

impl<K> MoveMutSentinel<K> {
    #[inline(always)]
    pub fn mutated() -> MoveMutSentinel<K> {
        MoveMutSentinel {
            self_ptr: Cell::new(NonNull::dangling()),
            _key: PhantomData
        }
    }

    #[inline(always)]
    pub fn was_moved_or_mutated(&self) -> bool {
        self.self_ptr.get().as_ptr() as *const MoveMutSentinel<K> != self as *const MoveMutSentinel<K>
    }

    #[inline(always)]
    pub fn set_mutated(&self) {
        self.self_ptr.set(Self::mutated().self_ptr.get())
    }

    #[inline(always)]
    pub fn set_unmutated(&self, _key: K) {
        self.self_ptr.set(unsafe{ NonNull::new_unchecked(self as *const Self as *mut Self) })
    }

    pub fn change_key<NK>(self) -> MoveMutSentinel<NK> {
        MoveMutSentinel {
            self_ptr: Cell::new(self.self_ptr.get().cast()),
            _key: PhantomData
        }
    }
}

impl<T, K> Deref for MoveMutTracker<T, K> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T, K> DerefMut for MoveMutTracker<T, K> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        self.sentinel.set_mutated();
        &mut self.value
    }
}

impl<T: Clone, K> Clone for MoveMutTracker<T, K> {
    #[inline(always)]
    fn clone(&self) -> MoveMutTracker<T, K> {
        MoveMutTracker::new(self.value.clone())
    }

    #[inline(always)]
    fn clone_from(&mut self, source: &Self) {
        self.sentinel.set_mutated();
        self.value.clone_from(source);
    }
}

impl<T, K> From<T> for MoveMutTracker<T, K> {
    #[inline(always)]
    fn from(t: T) -> MoveMutTracker<T, K> {
        MoveMutTracker::new(t)
    }
}

impl<K> fmt::Debug for MoveMutSentinel<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.was_moved_or_mutated() {
            true => write!(f, "MovedOrmutated"),
            false => write!(f, "Unmutated")
        }
    }
}

impl<T: fmt::Debug, K> fmt::Debug for MoveMutTracker<T, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MoveMutTracker")
            .field("sentinel", &self.sentinel)
            .field("value", &self.value)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    struct Key;

    #[test]
    fn moved() {
        let tracker: MoveMutTracker<_, Key> = MoveMutTracker::new(0);
        assert_eq!(true, MoveMutTracker::was_moved_or_mutated(&tracker));

        MoveMutTracker::set_unmutated(&tracker, Key);
        assert_eq!(false, MoveMutTracker::was_moved_or_mutated(&tracker));

        let tracker_on_heap = Box::new(tracker);
        assert_eq!(true, MoveMutTracker::was_moved_or_mutated(&tracker_on_heap));
    }

    #[test]
    fn mutated() {
        let mut tracker: MoveMutTracker<_, Key> = MoveMutTracker::new(0);
        assert_eq!(true, MoveMutTracker::was_moved_or_mutated(&tracker));

        MoveMutTracker::set_unmutated(&tracker, Key);
        assert_eq!(false, MoveMutTracker::was_moved_or_mutated(&tracker));

        *tracker = 1;
        assert_eq!(true, MoveMutTracker::was_moved_or_mutated(&tracker));
    }
}
