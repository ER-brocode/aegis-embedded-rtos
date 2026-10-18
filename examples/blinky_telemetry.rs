//! Blinky and Telemetry Task Example for Aegis RTOS
//!
//! Demonstrates multi-task initialization with GPIO toggling and UART telemetry transmission.

use aegis_rtos::hal::gpio::{GpioPin, PinMode};
use aegis_rtos::hal::uart::{MockUart, UartConfig};
use aegis_rtos::kernel::arch::MockArch;
use aegis_rtos::kernel::scheduler::{Scheduler, TaskPriority};
use embedded_hal::digital::OutputPin;

fn main() {
    let mut sched = Scheduler::<MockArch>::new();
    let mut led = GpioPin::new(13, PinMode::OutputPushPull);
    let uart = MockUart::new(UartConfig::default());

    // Register tasks
    let task_blink = sched.register_task_raw(TaskPriority::NORMAL, 0x2000_1000).unwrap();
    let task_telemetry = sched.register_task_raw(TaskPriority::HIGH, 0x2000_2000).unwrap();

    let _ = led.set_high();
    let _ = uart.write_all(b"[AEGIS RTOS] System Initialized.\r\n");

    // Execute scheduler steps
    sched.switch_context();
    sched.on_systick();

    let _ = (task_blink, task_telemetry);
}
