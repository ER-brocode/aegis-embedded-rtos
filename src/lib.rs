#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

pub mod hal;
pub mod kernel;
pub mod sync;

pub use kernel::{
    scheduler::{Scheduler, Task, TaskId, TaskPriority, TaskState, MAX_TASKS},
    time::Ticks,
};
pub use sync::{
    mutex::{Mutex, MutexGuard},
    ring_buffer::{RingBuffer, RingBufferConsumer, RingBufferProducer},
    semaphore::Semaphore,
};

/// RTOS Kernel version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Microsecond to tick conversion constant based on default 1000 Hz SysTick (1 ms per tick)
pub const TICKS_PER_SECOND: u32 = 1000;

#[inline]
pub const fn ms_to_ticks(ms: u32) -> u32 {
    ms * (TICKS_PER_SECOND / 1000)
}
