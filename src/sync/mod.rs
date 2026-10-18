//! Aegis RTOS Synchronization Primitives
//!
//! Provides thread-safe, interrupt-safe synchronization primitives for #![no_std]:
//! - Mutual Exclusion Locks (Mutex) with critical section RAII guard
//! - Counting and Binary Semaphores
//! - Lock-free single-producer single-consumer (SPSC) ring buffer for high-throughput ISR-to-Thread streaming

pub mod mutex;
pub mod ring_buffer;
pub mod semaphore;

pub use mutex::{Mutex, MutexGuard};
pub use ring_buffer::{RingBuffer, RingBufferConsumer, RingBufferProducer};
pub use semaphore::Semaphore;
