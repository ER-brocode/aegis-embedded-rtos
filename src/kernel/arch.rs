//! Architecture-specific context switching primitives and CPU abstraction.

use super::context::TaskContext;

pub trait ArchContext {
    /// Request an asynchronous context switch (e.g. PendSV trigger on Cortex-M)
    fn trigger_context_switch();

    /// Disable global interrupts and enter a critical section
    fn enter_critical() -> CriticalSectionToken;

    /// Restore interrupts using the critical section token
    fn exit_critical(token: CriticalSectionToken);

    /// Wait for interrupt / idle sleep
    fn wait_for_interrupt();
}

pub struct CriticalSectionToken {
    pub(crate) primask: u32,
}

#[cfg(feature = "cortex-m-arch")]
pub struct CortexMArch;

#[cfg(feature = "cortex-m-arch")]
impl ArchContext for CortexMArch {
    #[inline(always)]
    fn trigger_context_switch() {
        cortex_m::peripheral::SCB::set_pendsv();
    }

    #[inline(always)]
    fn enter_critical() -> CriticalSectionToken {
        let primask = cortex_m::register::primask::read().is_active() as u32;
        cortex_m::interrupt::disable();
        CriticalSectionToken { primask }
    }

    #[inline(always)]
    fn exit_critical(token: CriticalSectionToken) {
        if token.primask == 0 {
            unsafe {
                cortex_m::interrupt::enable();
            }
        }
    }

    #[inline(always)]
    fn wait_for_interrupt() {
        cortex_m::asm::wfi();
    }
}

/// Simulated architecture context for host machine unit testing and non-Cortex-M targets
pub struct MockArch;

static mut MOCK_SWITCH_REQUESTED: bool = false;
static mut MOCK_INTERRUPT_LEVEL: u32 = 0;

impl ArchContext for MockArch {
    fn trigger_context_switch() {
        unsafe {
            MOCK_SWITCH_REQUESTED = true;
        }
    }

    fn enter_critical() -> CriticalSectionToken {
        unsafe {
            let level = MOCK_INTERRUPT_LEVEL;
            MOCK_INTERRUPT_LEVEL += 1;
            CriticalSectionToken { primask: level }
        }
    }

    fn exit_critical(token: CriticalSectionToken) {
        unsafe {
            MOCK_INTERRUPT_LEVEL = token.primask;
        }
    }

    fn wait_for_interrupt() {
        // Mock idle hook
    }
}

impl MockArch {
    pub fn is_switch_requested() -> bool {
        unsafe { MOCK_SWITCH_REQUESTED }
    }

    pub fn clear_switch_request() {
        unsafe {
            MOCK_SWITCH_REQUESTED = false;
        }
    }

    pub fn interrupt_level() -> u32 {
        unsafe { MOCK_INTERRUPT_LEVEL }
    }
}
