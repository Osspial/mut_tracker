use std::ptr::NonNull;
use std::cell::Cell;
use std::marker::PhantomData;
use std::{fmt, mem};
use std::num::NonZeroUsize;

pub struct MoveMutSentinel<K> {
    self_ptr: Cell<NonNull<MoveMutSentinel<K>>>,
    _key: PhantomData<K>
}

// Repr C used to guarantee `key` comes after `anchor_key`, so &self - &key never equals zero.
#[repr(C)]
#[derive(Clone)]
pub struct MoveRelMutSentinel<K: PartialEq + Copy> {
    anchor_key_offset: Cell<NonZeroUsize>,
    key: Cell<K>
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

impl<K: PartialEq + Copy> MoveRelMutSentinel<K> {
    #[inline(always)]
    fn offset_of(&self, ref_key: &K) -> NonZeroUsize {
        let offset_isize = &self.anchor_key_offset as *const _ as isize - ref_key as *const _ as isize;
        NonZeroUsize::new(offset_isize as usize).unwrap()
    }

    #[inline(always)]
    fn self_offset(&self) -> NonZeroUsize {
        let offset_isize = &self.anchor_key_offset as *const _ as isize - &self.key as *const _ as isize;
        NonZeroUsize::new(offset_isize as usize).unwrap()
    }

    #[inline(always)]
    pub fn mutated() -> MoveRelMutSentinel<K> {
        let s = MoveRelMutSentinel {
            anchor_key_offset: Cell::new(NonZeroUsize::new(1).unwrap()),
            key: Cell::new(unsafe{ mem::uninitialized() })
        };
        let offset = s.self_offset();
        s.anchor_key_offset.set(offset);
        s
    }

    #[inline(always)]
    pub fn was_moved_or_mutated(&self, key: &K) -> bool {
        if self.anchor_key_offset.get() != self.offset_of(key) {
            true
        } else {
            // Placed in if/else to avoid reading uninitialized memory.
            self.key.get() != *key
        }
    }

    #[inline(always)]
    pub fn set_mutated(&self) {
        self.anchor_key_offset.set(self.self_offset());
    }

    #[inline(always)]
    pub fn set_unmutated(&self, key: &K) {
        self.key.set(*key);
        self.anchor_key_offset.set(self.offset_of(key));
    }
}

impl<K> Clone for MoveMutSentinel<K> {
    #[inline(always)]
    fn clone(&self) -> MoveMutSentinel<K> {
        MoveMutSentinel {
            self_ptr: self.self_ptr.clone(),
            _key: PhantomData
        }
    }
}

impl<K> fmt::Debug for MoveMutSentinel<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.was_moved_or_mutated() {
            true => write!(f, "MovedOrMutated"),
            false => write!(f, "Unmutated")
        }
    }
}
