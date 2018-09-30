mod sentinel;
use sentinel::*;

use std::ops::{Deref, DerefMut};
use std::fmt;


pub struct MoveMutTracker<T, K> {
    value: T,
    sentinel: MoveMutSentinel<K>
}

pub struct MoveRelMutTracker<T, K: PartialEq + Copy> {
    value: T,
    sentinel: MoveRelMutSentinel<K>
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

impl<T, K: PartialEq + Copy> MoveRelMutTracker<T, K> {
    #[inline(always)]
    pub fn new(value: T) -> MoveRelMutTracker<T, K> {
        MoveRelMutTracker {
            value,
            sentinel: MoveRelMutSentinel::mutated()
        }
    }

    #[inline(always)]
    pub fn was_moved_or_mutated(this: &Self, key: &K) -> bool {
        this.sentinel.was_moved_or_mutated(key)
    }

    #[inline(always)]
    pub fn set_unmutated(this: &Self, key: &K) {
        this.sentinel.set_unmutated(key);
    }
}

macro_rules! impl_deref {
    ($tracker:ident$({K: $($t:tt)+})*) => {
        impl<T, K $(: $($t)+)*> Deref for $tracker<T, K> {
            type Target = T;

            #[inline(always)]
            fn deref(&self) -> &T {
                &self.value
            }
        }

        impl<T, K $(: $($t)+)*> DerefMut for $tracker<T, K> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut T {
                self.sentinel.set_mutated();
                &mut self.value
            }
        }

    };
}

impl_deref!(MoveMutTracker);
impl_deref!(MoveRelMutTracker{K: PartialEq + Copy});

impl<T, K> From<T> for MoveMutTracker<T, K> {
    #[inline(always)]
    fn from(t: T) -> MoveMutTracker<T, K> {
        MoveMutTracker::new(t)
    }
}

impl<T: Clone, K> Clone for MoveMutTracker<T, K> {
    #[inline(always)]
    fn clone(&self) -> MoveMutTracker<T, K> {
        MoveMutTracker {
            value: self.value.clone(),
            sentinel: self.sentinel.clone()
        }
    }

    #[inline(always)]
    fn clone_from(&mut self, source: &Self) {
        self.sentinel.set_mutated();
        self.value.clone_from(source);
    }
}

impl<T: Clone, K: PartialEq + Copy> Clone for MoveRelMutTracker<T, K> {
    #[inline(always)]
    fn clone(&self) -> MoveRelMutTracker<T, K> {
        MoveRelMutTracker {
            value: self.value.clone(),
            sentinel: self.sentinel.clone()
        }
    }

    #[inline(always)]
    fn clone_from(&mut self, source: &Self) {
        self.sentinel.set_mutated();
        self.value.clone_from(source);
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

impl<T: fmt::Debug, K: PartialEq + Copy> fmt::Debug for MoveRelMutTracker<T, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MoveMutTracker")
            .field("value", &self.value)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    struct Key;

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct KeyInt(i32);

    struct Container {
        key: KeyInt,
        tracker: MoveRelMutTracker<u32, KeyInt>
    }

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

    #[test]
    fn moved_rel() {
        let container0 = Container {
            key: KeyInt(897),
            tracker: MoveRelMutTracker::new(1)
        };
        assert_eq!(true, MoveRelMutTracker::was_moved_or_mutated(&container0.tracker, &container0.key));

        MoveRelMutTracker::set_unmutated(&container0.tracker, &container0.key);
        assert_eq!(false, MoveRelMutTracker::was_moved_or_mutated(&container0.tracker, &container0.key));

        let mut container0 = Box::new(container0);
        assert_eq!(false, MoveRelMutTracker::was_moved_or_mutated(&container0.tracker, &container0.key));


        let mut container1 = Container {
            key: KeyInt(231),
            tracker: MoveRelMutTracker::new(2)
        };
        assert_eq!(true, MoveRelMutTracker::was_moved_or_mutated(&container1.tracker, &container1.key));

        MoveRelMutTracker::set_unmutated(&container1.tracker, &container1.key);
        assert_eq!(false, MoveRelMutTracker::was_moved_or_mutated(&container1.tracker, &container1.key));

        ::std::mem::swap(&mut container0.tracker, &mut container1.tracker);
        assert_eq!(true, MoveRelMutTracker::was_moved_or_mutated(&container0.tracker, &container0.key));
        assert_eq!(true, MoveRelMutTracker::was_moved_or_mutated(&container1.tracker, &container1.key));
    }

    #[test]
    fn mutated_rel() {
        let mut container = Container {
            key: KeyInt(3243),
            tracker: MoveRelMutTracker::new(0)
        };
        assert_eq!(true, MoveRelMutTracker::was_moved_or_mutated(&container.tracker, &container.key));

        MoveRelMutTracker::set_unmutated(&container.tracker, &container.key);
        assert_eq!(false, MoveRelMutTracker::was_moved_or_mutated(&container.tracker, &container.key));

        *container.tracker = 1;
        assert_eq!(true, MoveRelMutTracker::was_moved_or_mutated(&container.tracker, &container.key));
    }
}
