//! Context switching representation and hardware/mock stack frames.

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackFrame {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32,
    pub pc: u32,
    pub xpsr: u32,
}

impl Default for StackFrame {
    fn default() -> Self {
        Self {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r12: 0,
            lr: 0xFFFF_FFF9, // EXC_RETURN: Return to Thread mode, exception using Main Stack Pointer (MSP)
            pc: 0,
            xpsr: 0x0100_0000, // Default Thumb state bit set
        }
    }
}

/// Software-saved registers (Cortex-M callee-saved registers R4-R11)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CalleeSavedContext {
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
}

/// Full execution context container for a task
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TaskContext {
    pub stack_ptr: usize,
    pub callee_saved: CalleeSavedContext,
    pub exception_frame: StackFrame,
}

impl TaskContext {
    pub const fn empty() -> Self {
        Self {
            stack_ptr: 0,
            callee_saved: CalleeSavedContext {
                r4: 0,
                r5: 0,
                r6: 0,
                r7: 0,
                r8: 0,
                r9: 0,
                r10: 0,
                r11: 0,
            },
            exception_frame: StackFrame {
                r0: 0,
                r1: 0,
                r2: 0,
                r3: 0,
                r12: 0,
                lr: 0xFFFF_FFF9,
                pc: 0,
                xpsr: 0x0100_0000,
            },
        }
    }

    /// Pre-populates a task's initial stack slice so that returning from an exception
    /// or restoring callee registers naturally executes `entry_point(arg)`.
    ///
    /// The stack grows downward. `stack` is a mutable slice representing task stack memory.
    pub fn initialize_stack(
        stack: &mut [u8],
        entry_point: extern "C" fn(usize) -> !,
        arg: usize,
    ) -> Result<usize, &'static str> {
        let len = stack.len();
        if len < 64 {
            return Err("Stack buffer too small (minimum 64 bytes)");
        }

        // Align top of stack to 8-byte boundary per AAPCS (ARM Architecture Procedure Call Standard)
        let stack_base = stack.as_mut_ptr() as usize;
        let stack_top = (stack_base + len) & !0x7;

        // Offset downward to place hardware exception frame (8 * 4 = 32 bytes)
        // and software callee-saved registers (8 * 4 = 32 bytes)
        let total_frame_size = 64;
        let initial_sp = stack_top - total_frame_size;

        if initial_sp < stack_base {
            return Err("Stack overflow during initialization");
        }

        unsafe {
            // Callee-saved context starts at initial_sp
            let callee_ptr = initial_sp as *mut u32;
            for i in 0..8 {
                *callee_ptr.add(i) = 0; // r4 - r11 = 0
            }

            // Hardware stack frame starts at initial_sp + 32
            let hw_ptr = (initial_sp + 32) as *mut u32;
            *hw_ptr.add(0) = arg as u32;         // r0 (argument)
            *hw_ptr.add(1) = 0;                  // r1
            *hw_ptr.add(2) = 0;                  // r2
            *hw_ptr.add(3) = 0;                  // r3
            *hw_ptr.add(4) = 0;                  // r12
            *hw_ptr.add(5) = 0xFFFF_FFF9;        // lr (EXC_RETURN)
            *hw_ptr.add(6) = entry_point as u32; // pc
            *hw_ptr.add(7) = 0x0100_0000;        // xPSR: Thumb bit set
        }

        Ok(initial_sp)
    }
}
