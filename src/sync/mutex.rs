//! Mutual Exclusion Primitive (Mutex) with RAII Guard
//!
//! Provides deterministic critical-section mutual exclusion for embedded platforms.
//! Prevents priority inversion and data corruption across concurrent task executions.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use crate::kernel::arch::{ArchContext, CriticalSectionToken};

pub struct Mutex<A: ArchContext, T> {
    data: UnsafeCell<T>,
    _arch: core::marker::PhantomData<A>,
}

unsafe impl<A: ArchContext, T: Send> Sync for Mutex<A, T> {}
unsafe impl<A: ArchContext, T: Send> Send for Mutex<A, T> {}

impl<A: ArchContext, T> Mutex<A, T> {
    pub const fn new(value: T) -> Self {
        Self {
            data: UnsafeCell::new(value),
            _arch: core::marker::PhantomData,
        }
    }

    /// Acquires the lock by entering an architecture critical section.
    /// Returns an RAII `MutexGuard` that restores the critical section on drop.
    pub fn lock(&self) -> MutexGuard<'_, A, T> {
        let token = A::enter_critical();
        MutexGuard {
            mutex: self,
            token,
        }
    }

    /// Consumes the mutex and yields the underlying data
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

pub struct MutexGuard<'a, A: ArchContext, T> {
    mutex: &'a Mutex<A, T>,
    token: CriticalSectionToken,
}

impl<'a, A: ArchContext, T> Deref for MutexGuard<'a, A, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, A: ArchContext, T> DerefMut for MutexGuard<'a, A, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, A: ArchContext, T> Drop for MutexGuard<'a, A, T> {
    fn drop(&mut self) {
        A::exit_critical(CriticalSectionToken { primask: self.token.primask });
    }
}
