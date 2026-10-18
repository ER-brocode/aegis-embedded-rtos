//! Counting and Binary Semaphore Synchronization Primitive

use core::sync::atomic::{AtomicIsize, Ordering};
use crate::kernel::arch::ArchContext;

pub struct Semaphore<A: ArchContext> {
    count: AtomicIsize,
    max_count: isize,
    _arch: core::marker::PhantomData<A>,
}

impl<A: ArchContext> Semaphore<A> {
    pub const fn new(initial: isize, max: isize) -> Self {
        Self {
            count: AtomicIsize::new(initial),
            max_count: max,
            _arch: core::marker::PhantomData,
        }
    }

    pub const fn binary(initial_available: bool) -> Self {
        Self::new(if initial_available { 1 } else { 0 }, 1)
    }

    /// Try to acquire a permit non-blockingly.
    /// Returns true if permit was acquired, false otherwise.
    pub fn try_acquire(&self) -> bool {
        let mut current = self.count.load(Ordering::Relaxed);
        loop {
            if current <= 0 {
                return false;
            }
            match self.count.compare_exchange_weak(
                current,
                current - 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    /// Release a permit back to the semaphore.
    /// Returns Ok(()) on success, or Err("Exceeded max count") if saturated.
    pub fn release(&self) -> Result<(), &'static str> {
        let mut current = self.count.load(Ordering::Relaxed);
        loop {
            if current >= self.max_count {
                return Err("Semaphore capacity exceeded");
            }
            match self.count.compare_exchange_weak(
                current,
                current + 1,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(actual) => current = actual,
            }
        }
    }

    /// Get current permits count
    pub fn value(&self) -> isize {
        self.count.load(Ordering::Relaxed)
    }
}
