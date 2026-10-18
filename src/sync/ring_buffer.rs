//! Lock-Free Single-Producer Single-Consumer (SPSC) Ring Buffer
//!
//! Designed specifically for interrupt service routines (ISR) pushing data into thread context
//! without disabling global interrupts or incurring lock contention.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct RingBuffer<T, const CAP: usize> {
    buffer: [UnsafeCell<Option<T>>; CAP],
    head: AtomicUsize, // Written by Producer
    tail: AtomicUsize, // Written by Consumer
}

unsafe impl<T: Send, const CAP: usize> Sync for RingBuffer<T, CAP> {}
unsafe impl<T: Send, const CAP: usize> Send for RingBuffer<T, CAP> {}

impl<T: Copy, const CAP: usize> RingBuffer<T, CAP> {
    const INIT: UnsafeCell<Option<T>> = UnsafeCell::new(None);

    pub const fn new() -> Self {
        assert!(CAP > 0, "Capacity must be greater than zero");
        Self {
            buffer: [Self::INIT; CAP],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Try to push an item into the ring buffer.
    /// Returns Err(item) if the buffer is full.
    pub fn push(&self, item: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        let next_head = (head + 1) % CAP;
        if next_head == tail {
            // Buffer full
            return Err(item);
        }

        unsafe {
            *self.buffer[head].get() = Some(item);
        }

        self.head.store(next_head, Ordering::Release);
        Ok(())
    }

    /// Try to pop an item from the ring buffer.
    /// Returns None if the buffer is empty.
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if head == tail {
            // Buffer empty
            return None;
        }

        let item = unsafe { (*self.buffer[tail].get()).take() };

        let next_tail = (tail + 1) % CAP;
        self.tail.store(next_tail, Ordering::Release);

        item
    }

    /// Current number of elements stored in the ring buffer
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        if head >= tail {
            head - tail
        } else {
            CAP - (tail - head)
        }
    }

    /// Checks if ring buffer is empty
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }

    /// Checks if ring buffer is full
    pub fn is_full(&self) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        (head + 1) % CAP == tail
    }

    /// Available capacity
    pub fn capacity(&self) -> usize {
        CAP - 1
    }

    /// Split into producer and consumer endpoints
    pub fn split(&'static self) -> (RingBufferProducer<'static, T, CAP>, RingBufferConsumer<'static, T, CAP>) {
        (
            RingBufferProducer { rb: self },
            RingBufferConsumer { rb: self },
        )
    }
}

pub struct RingBufferProducer<'a, T: Copy, const CAP: usize> {
    rb: &'a RingBuffer<T, CAP>,
}

impl<'a, T: Copy, const CAP: usize> RingBufferProducer<'a, T, CAP> {
    #[inline(always)]
    pub fn push(&mut self, item: T) -> Result<(), T> {
        self.rb.push(item)
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.rb.is_full()
    }
}

pub struct RingBufferConsumer<'a, T: Copy, const CAP: usize> {
    rb: &'a RingBuffer<T, CAP>,
}

impl<'a, T: Copy, const CAP: usize> RingBufferConsumer<'a, T, CAP> {
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        self.rb.pop()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.rb.is_empty()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.rb.len()
    }
}
