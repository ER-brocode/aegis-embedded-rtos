//! Aegis RTOS Core Kernel Architecture
//!
//! Provides preemptive priority-based task scheduling, execution context switching primitives,
//! stack layout management, and tick timer accounting.

pub mod arch;
pub mod context;
pub mod scheduler;
pub mod time;

pub use arch::ArchContext;
pub use context::{StackFrame, TaskContext};
pub use scheduler::{Scheduler, Task, TaskControlBlock, TaskId, TaskPriority, TaskState, MAX_TASKS};
pub use time::{SysTick, Ticks};
