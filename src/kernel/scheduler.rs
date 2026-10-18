//! Preemptive Priority-Based Task Scheduler
//!
//! Features:
//! - Priority-driven preemptive scheduling (highest priority executes first)
//! - Round-robin time-slicing among tasks with identical priority
//! - Non-blocking deterministic tick handling
//! - Sleep / Delay wake-up mechanics
//! - Zero dynamic heap allocation (#![no_std] fixed array capacity)

use super::arch::ArchContext;
use super::context::TaskContext;
use super::time::{SysTick, Ticks};

pub const MAX_TASKS: usize = 16;
pub const IDLE_TASK_ID: TaskId = TaskId(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskPriority(pub u8);

impl TaskPriority {
    pub const IDLE: TaskPriority = TaskPriority(0);
    pub const LOW: TaskPriority = TaskPriority(1);
    pub const NORMAL: TaskPriority = TaskPriority(10);
    pub const HIGH: TaskPriority = TaskPriority(20);
    pub const CRITICAL: TaskPriority = TaskPriority(31);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Unused,
    Ready,
    Running,
    Blocked,
    Sleeping(Ticks),
    Terminated,
}

#[derive(Debug, Clone, Copy)]
pub struct TaskControlBlock {
    pub id: TaskId,
    pub priority: TaskPriority,
    pub state: TaskState,
    pub stack_ptr: usize,
    pub stack_base: usize,
    pub stack_size: usize,
    pub runtime_ticks: u32,
    pub context: TaskContext,
}

impl TaskControlBlock {
    pub const fn empty() -> Self {
        Self {
            id: TaskId(0),
            priority: TaskPriority::IDLE,
            state: TaskState::Unused,
            stack_ptr: 0,
            stack_base: 0,
            stack_size: 0,
            runtime_ticks: 0,
            context: TaskContext::empty(),
        }
    }
}

pub struct Task {
    pub id: TaskId,
    pub priority: TaskPriority,
}

pub struct Scheduler<A: ArchContext> {
    tasks: [TaskControlBlock; MAX_TASKS],
    current_task: Option<TaskId>,
    task_count: usize,
    systick: SysTick,
    _arch: core::marker::PhantomData<A>,
}

impl<A: ArchContext> Scheduler<A> {
    pub const fn new() -> Self {
        Self {
            tasks: [TaskControlBlock::empty(); MAX_TASKS],
            current_task: None,
            task_count: 0,
            systick: SysTick::new(),
            _arch: core::marker::PhantomData,
        }
    }

    /// Spawns a task into the scheduler table.
    pub fn spawn(
        &mut self,
        priority: TaskPriority,
        stack: &'static mut [u8],
        entry: extern "C" fn(usize) -> !,
        arg: usize,
    ) -> Result<TaskId, &'static str> {
        let token = A::enter_critical();

        if self.task_count >= MAX_TASKS {
            A::exit_critical(token);
            return Err("Maximum task limit reached");
        }

        // Find unused slot
        let slot = self
            .tasks
            .iter()
            .position(|t| t.state == TaskState::Unused)
            .ok_or("No free task slot")?;

        let initial_sp = TaskContext::initialize_stack(stack, entry, arg)?;
        let id = TaskId(slot);

        self.tasks[slot] = TaskControlBlock {
            id,
            priority,
            state: TaskState::Ready,
            stack_ptr: initial_sp,
            stack_base: stack.as_ptr() as usize,
            stack_size: stack.len(),
            runtime_ticks: 0,
            context: TaskContext {
                stack_ptr: initial_sp,
                ..TaskContext::empty()
            },
        };

        self.task_count += 1;

        // If the newly spawned task has higher priority than currently running task, request switch
        if let Some(curr) = self.current_task {
            if priority > self.tasks[curr.0].priority {
                A::trigger_context_switch();
            }
        }

        A::exit_critical(token);
        Ok(id)
    }

    /// Directly register a pre-allocated task with an already initialized stack pointer (useful in mock tests)
    pub fn register_task_raw(
        &mut self,
        priority: TaskPriority,
        stack_ptr: usize,
    ) -> Result<TaskId, &'static str> {
        let token = A::enter_critical();

        let slot = self
            .tasks
            .iter()
            .position(|t| t.state == TaskState::Unused)
            .ok_or("No free task slot")?;

        let id = TaskId(slot);
        self.tasks[slot] = TaskControlBlock {
            id,
            priority,
            state: TaskState::Ready,
            stack_ptr,
            stack_base: stack_ptr,
            stack_size: 1024,
            runtime_ticks: 0,
            context: TaskContext {
                stack_ptr,
                ..TaskContext::empty()
            },
        };
        self.task_count += 1;

        A::exit_critical(token);
        Ok(id)
    }

    /// SysTick handler to be called from timer ISR.
    /// Advances system time, wakes sleeping tasks, and checks preemption.
    pub fn on_systick(&mut self) -> bool {
        let token = A::enter_critical();
        let current_ticks = self.systick.increment();

        // Increment runtime ticks for running task
        if let Some(curr) = self.current_task {
            self.tasks[curr.0].runtime_ticks = self.tasks[curr.0].runtime_ticks.saturating_add(1);
        }

        let mut need_switch = false;

        // Wake up sleeping tasks
        for task in self.tasks.iter_mut() {
            if let TaskState::Sleeping(wake_tick) = task.state {
                if current_ticks >= wake_tick {
                    task.state = TaskState::Ready;
                    if let Some(curr) = self.current_task {
                        if task.priority >= self.tasks[curr.0].priority {
                            need_switch = true;
                        }
                    } else {
                        need_switch = true;
                    }
                }
            }
        }

        if need_switch {
            A::trigger_context_switch();
        }

        A::exit_critical(token);
        need_switch
    }

    /// Primary scheduling algorithm: returns next TaskId to run
    pub fn select_next_task(&mut self) -> Option<TaskId> {
        let mut highest_priority = TaskPriority(0);
        let mut candidate: Option<TaskId> = None;

        // Find the maximum priority among Ready/Running tasks
        for task in self.tasks.iter() {
            if task.state == TaskState::Ready || task.state == TaskState::Running {
                if task.priority > highest_priority || candidate.is_none() {
                    highest_priority = task.priority;
                    candidate = Some(task.id);
                }
            }
        }

        // Round-robin tie breaking: If multiple tasks share the highest priority,
        // select the next ready one after the current task index
        let curr_idx = self.current_task.map(|t| t.0).unwrap_or(0);
        for i in 1..=MAX_TASKS {
            let idx = (curr_idx + i) % MAX_TASKS;
            let task = &self.tasks[idx];
            if (task.state == TaskState::Ready || task.state == TaskState::Running)
                && task.priority == highest_priority
            {
                candidate = Some(task.id);
                break;
            }
        }

        candidate
    }

    /// Executes context switch transition
    pub fn switch_context(&mut self) -> Option<(TaskId, TaskId)> {
        let token = A::enter_critical();
        let next_id = self.select_next_task();

        let result = match (self.current_task, next_id) {
            (Some(curr), Some(next)) if curr != next => {
                if self.tasks[curr.0].state == TaskState::Running {
                    self.tasks[curr.0].state = TaskState::Ready;
                }
                self.tasks[next.0].state = TaskState::Running;
                self.current_task = Some(next);
                Some((curr, next))
            }
            (None, Some(next)) => {
                self.tasks[next.0].state = TaskState::Running;
                self.current_task = Some(next);
                Some((next, next))
            }
            _ => None,
        };

        A::exit_critical(token);
        result
    }

    /// Puts current task to sleep for duration in ticks
    pub fn sleep(&mut self, duration: Ticks) {
        let token = A::enter_critical();
        if let Some(curr) = self.current_task {
            let wake_tick = self.systick.get().wrapping_add(duration);
            self.tasks[curr.0].state = TaskState::Sleeping(wake_tick);
            A::trigger_context_switch();
        }
        A::exit_critical(token);
    }

    /// Yield current execution slice to other tasks of same/higher priority
    pub fn yield_now(&mut self) {
        let token = A::enter_critical();
        if let Some(curr) = self.current_task {
            self.tasks[curr.0].state = TaskState::Ready;
            A::trigger_context_switch();
        }
        A::exit_critical(token);
    }

    /// Get current running task ID
    pub fn current_task_id(&self) -> Option<TaskId> {
        self.current_task
    }

    /// Get current ticks
    pub fn ticks(&self) -> Ticks {
        self.systick.get()
    }

    /// Get TaskControlBlock by ID
    pub fn task_tcb(&self, id: TaskId) -> Option<&TaskControlBlock> {
        if id.0 < MAX_TASKS && self.tasks[id.0].state != TaskState::Unused {
            Some(&self.tasks[id.0])
        } else {
            None
        }
    }

    /// Mark a task blocked (e.g. waiting for mutex/semaphore)
    pub fn block_task(&mut self, id: TaskId) {
        let token = A::enter_critical();
        if id.0 < MAX_TASKS {
            self.tasks[id.0].state = TaskState::Blocked;
            if self.current_task == Some(id) {
                A::trigger_context_switch();
            }
        }
        A::exit_critical(token);
    }

    /// Unblock a previously blocked task
    pub fn unblock_task(&mut self, id: TaskId) {
        let token = A::enter_critical();
        if id.0 < MAX_TASKS && self.tasks[id.0].state == TaskState::Blocked {
            self.tasks[id.0].state = TaskState::Ready;
            if let Some(curr) = self.current_task {
                if self.tasks[id.0].priority > self.tasks[curr.0].priority {
                    A::trigger_context_switch();
                }
            }
        }
        A::exit_critical(token);
    }
}
