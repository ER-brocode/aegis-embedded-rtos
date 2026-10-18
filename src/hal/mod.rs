//! Hardware Abstraction Layer (HAL)
//!
//! Provides clean, type-safe peripheral drivers compatible with `embedded-hal` v1.0.0 traits:
//! - GPIO (General Purpose Input/Output: Input, Output, Push-Pull, Open-Drain)
//! - UART (Serial asynchronous communication: Transmit & Receive)
//! - SPI (Serial Peripheral Interface master bus operations)

pub mod gpio;
pub mod spi;
pub mod uart;

pub use gpio::{GpioPin, OutputState, PinMode};
pub use spi::{MockSpiBus, SpiConfig, SpiMode};
pub use uart::{MockUart, UartConfig, UartParity};
