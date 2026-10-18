//! UART (Universal Asynchronous Receiver-Transmitter) HAL
//!
//! Provides buffered, interrupt-compatible UART communication with embedded ring buffers.

use crate::sync::ring_buffer::RingBuffer;

pub const UART_RX_BUFFER_SIZE: usize = 128;
pub const UART_TX_BUFFER_SIZE: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UartParity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UartConfig {
    pub baudrate: u32,
    pub parity: UartParity,
    pub stop_bits: u8,
    pub data_bits: u8,
}

impl Default for UartConfig {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            parity: UartParity::None,
            stop_bits: 1,
            data_bits: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UartError {
    TxBufferFull,
    RxBufferEmpty,
    FramingError,
    ParityError,
    OverrunError,
}

pub struct MockUart {
    pub config: UartConfig,
    pub rx_buf: RingBuffer<u8, UART_RX_BUFFER_SIZE>,
    pub tx_buf: RingBuffer<u8, UART_TX_BUFFER_SIZE>,
}

impl MockUart {
    pub const fn new(config: UartConfig) -> Self {
        Self {
            config,
            rx_buf: RingBuffer::new(),
            tx_buf: RingBuffer::new(),
        }
    }

    /// Transmit a single byte by queuing it into the TX buffer
    pub fn write_byte(&self, byte: u8) -> Result<(), UartError> {
        self.tx_buf.push(byte).map_err(|_| UartError::TxBufferFull)
    }

    /// Read a single byte from the RX buffer
    pub fn read_byte(&self) -> Result<u8, UartError> {
        self.rx_buf.pop().ok_or(UartError::RxBufferEmpty)
    }

    /// Write a slice of bytes into the TX buffer
    pub fn write_all(&self, data: &[u8]) -> Result<usize, UartError> {
        let mut written = 0;
        for &b in data {
            if self.write_byte(b).is_err() {
                break;
            }
            written += 1;
        }
        if written < data.len() {
            Err(UartError::TxBufferFull)
        } else {
            Ok(written)
        }
    }

    /// ISR Hook: Simulates hardware RX interrupt putting byte into RX buffer
    pub fn isr_rx_received(&self, byte: u8) -> Result<(), u8> {
        self.rx_buf.push(byte)
    }

    /// ISR Hook: Simulates hardware TX interrupt extracting byte to transmit
    pub fn isr_tx_ready(&self) -> Option<u8> {
        self.tx_buf.pop()
    }
}
