//! RTOS Tick and Timer management

use core::sync::atomic::{AtomicU32, Ordering};

pub type Ticks = u32;

pub struct SysTick {
    ticks: AtomicU32,
}

impl SysTick {
    pub const fn new() -> Self {
        Self {
            ticks: AtomicU32::new(0),
        }
    }

    #[inline(always)]
    pub fn increment(&self) -> Ticks {
        self.ticks.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
    }

    #[inline(always)]
    pub fn get(&self) -> Ticks {
        self.ticks.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn reset(&self) {
        self.ticks.store(0, Ordering::Relaxed);
    }
}

impl Default for SysTick {
    fn default() -> Self {
        Self::new()
    }
}
