extern crate aegis_rtos;

use aegis_rtos::hal::gpio::{GpioPin, OutputState, PinMode};
use aegis_rtos::hal::spi::{MockSpiBus, SpiConfig, SpiMode};
use aegis_rtos::hal::uart::{MockUart, UartConfig};
use aegis_rtos::kernel::arch::{ArchContext, MockArch};
use aegis_rtos::kernel::scheduler::{Scheduler, TaskPriority, TaskState, MAX_TASKS};
use aegis_rtos::sync::mutex::Mutex;
use aegis_rtos::sync::ring_buffer::RingBuffer;
use aegis_rtos::sync::semaphore::Semaphore;
use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
use embedded_hal::spi::SpiBus;

#[test]
fn test_scheduler_preemptive_priority() {
    MockArch::clear_switch_request();
    let mut sched = Scheduler::<MockArch>::new();

    let low_task = sched
        .register_task_raw(TaskPriority::LOW, 0x2000_1000)
        .expect("Failed to register low task");
    let high_task = sched
        .register_task_raw(TaskPriority::HIGH, 0x2000_2000)
        .expect("Failed to register high task");

    // Initially high priority task should be selected
    let next = sched.select_next_task();
    assert_eq!(next, Some(high_task));

    let switch = sched.switch_context();
    assert_eq!(switch, Some((high_task, high_task)));
    assert_eq!(sched.current_task_id(), Some(high_task));

    // Put high task to sleep
    sched.sleep(10);
    assert!(MockArch::is_switch_requested());
    MockArch::clear_switch_request();

    // Now low priority task must run
    let next_after_sleep = sched.select_next_task();
    assert_eq!(next_after_sleep, Some(low_task));

    let switch2 = sched.switch_context();
    assert_eq!(switch2, Some((high_task, low_task)));
    assert_eq!(sched.current_task_id(), Some(low_task));

    // Advance 9 ticks: high task should still be sleeping
    for _ in 0..9 {
        sched.on_systick();
    }
    assert_eq!(sched.select_next_task(), Some(low_task));

    // Advance 10th tick: high task wakes up and preempts!
    let preempted = sched.on_systick();
    assert!(preempted);
    assert_eq!(sched.select_next_task(), Some(high_task));
}

#[test]
fn test_scheduler_round_robin() {
    let mut sched = Scheduler::<MockArch>::new();

    let t1 = sched.register_task_raw(TaskPriority::NORMAL, 0x2000_1000).unwrap();
    let t2 = sched.register_task_raw(TaskPriority::NORMAL, 0x2000_2000).unwrap();

    let _ = sched.switch_context();
    assert_eq!(sched.current_task_id(), Some(t1));

    // Next round-robin candidate should be t2
    let next = sched.select_next_task();
    assert_eq!(next, Some(t2));
}

#[test]
fn test_lock_free_ring_buffer() {
    let rb = RingBuffer::<u8, 8>::new();
    assert!(rb.is_empty());
    assert!(!rb.is_full());

    // Fill buffer up to capacity (CAP - 1 = 7)
    for i in 0..7 {
        assert!(rb.push(i).is_ok());
    }
    assert!(rb.is_full());
    assert_eq!(rb.push(99), Err(99)); // Overflow rejected

    // Pop elements and verify FIFO
    for i in 0..7 {
        assert_eq!(rb.pop(), Some(i));
    }
    assert!(rb.is_empty());
    assert_eq!(rb.pop(), None);
}

#[test]
fn test_mutex_critical_section() {
    let mutex = Mutex::<MockArch, u32>::new(42);
    {
        let mut guard = mutex.lock();
        assert_eq!(*guard, 42);
        *guard += 8;
    }
    assert_eq!(mutex.into_inner(), 50);
}

#[test]
fn test_counting_and_binary_semaphore() {
    let sem = Semaphore::<MockArch>::new(2, 2);
    assert!(sem.try_acquire());
    assert_eq!(sem.value(), 1);
    assert!(sem.try_acquire());
    assert_eq!(sem.value(), 0);
    assert!(!sem.try_acquire()); // Empty

    assert!(sem.release().is_ok());
    assert_eq!(sem.value(), 1);
    assert!(sem.release().is_ok());
    assert!(sem.release().is_err()); // Exceeded max
}

#[test]
fn test_hal_gpio() {
    let mut pin = GpioPin::new(13, PinMode::OutputPushPull);
    assert!(pin.set_high().is_ok());
    assert_eq!(pin.read_state(), OutputState::High);
    assert!(pin.is_set_high().unwrap());

    assert!(pin.set_low().is_ok());
    assert_eq!(pin.read_state(), OutputState::Low);
    assert!(pin.is_set_low().unwrap());
}

#[test]
fn test_hal_uart_ring_buffers() {
    let uart = MockUart::new(UartConfig::default());

    // Producer / TX test
    assert!(uart.write_byte(0x55).is_ok());
    assert_eq!(uart.isr_tx_ready(), Some(0x55));
    assert_eq!(uart.isr_tx_ready(), None);

    // Consumer / RX ISR test
    assert!(uart.isr_rx_received(0xAA).is_ok());
    assert_eq!(uart.read_byte().unwrap(), 0xAA);
    assert!(uart.read_byte().is_err());
}

#[test]
fn test_hal_spi_bus() {
    let mut spi = MockSpiBus::new(SpiConfig::default());
    let tx_data = [0x10, 0x20, 0x30];
    let mut rx_data = [0u8; 3];

    assert!(spi.transfer(&mut rx_data, &tx_data).is_ok());
    assert_eq!(&spi.last_written[..3], &tx_data);
    assert_eq!(rx_data, [0xA5, 0xA5, 0xA5]);
}
