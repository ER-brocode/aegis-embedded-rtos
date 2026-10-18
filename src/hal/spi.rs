//! SPI (Serial Peripheral Interface) HAL
//!
//! Provides embedded-hal SPI Bus trait implementation for microcontroller inter-chip communication.

use embedded_hal::spi::{ErrorType, SpiBus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    Mode0, // CPOL=0, CPHA=0
    Mode1, // CPOL=0, CPHA=1
    Mode2, // CPOL=1, CPHA=0
    Mode3, // CPOL=1, CPHA=1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiConfig {
    pub frequency_hz: u32,
    pub mode: SpiMode,
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 1_000_000,
            mode: SpiMode::Mode0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiError {
    BusFault,
    ModeFault,
    Overrun,
}

impl core::fmt::Display for SpiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SpiError::BusFault => write!(f, "SPI Bus Fault"),
            SpiError::ModeFault => write!(f, "SPI Mode Fault"),
            SpiError::Overrun => write!(f, "SPI Overrun Error"),
        }
    }
}

impl embedded_hal::spi::Error for SpiError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        embedded_hal::spi::ErrorKind::Other
    }
}

pub struct MockSpiBus {
    pub config: SpiConfig,
    pub last_written: [u8; 64],
    pub last_written_len: usize,
    pub rx_pattern: u8,
}

impl MockSpiBus {
    pub const fn new(config: SpiConfig) -> Self {
        Self {
            config,
            last_written: [0u8; 64],
            last_written_len: 0,
            rx_pattern: 0xA5,
        }
    }

    pub fn set_rx_pattern(&mut self, pattern: u8) {
        self.rx_pattern = pattern;
    }
}

impl ErrorType for MockSpiBus {
    type Error = SpiError;
}

impl SpiBus<u8> for MockSpiBus {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        for word in words.iter_mut() {
            *word = self.rx_pattern;
        }
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let copy_len = words.len().min(self.last_written.len());
        self.last_written[..copy_len].copy_from_slice(&words[..copy_len]);
        self.last_written_len = copy_len;
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.write(write)?;
        self.read(read)?;
        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.write(words)?;
        for word in words.iter_mut() {
            *word ^= self.rx_pattern;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
