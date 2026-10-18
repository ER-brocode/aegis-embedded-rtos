//! GPIO Hardware Abstraction Layer
//!
//! Provides embedded-hal digital Pin implementation with configurable input/output states.

use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    InputFloating,
    InputPullUp,
    InputPullDown,
    OutputPushPull,
    OutputOpenDrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputState {
    Low,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioError {
    InvalidMode,
    HardwareFault,
}

impl core::fmt::Display for GpioError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GpioError::InvalidMode => write!(f, "Invalid GPIO Mode for requested operation"),
            GpioError::HardwareFault => write!(f, "GPIO Hardware Fault"),
        }
    }
}

impl embedded_hal::digital::Error for GpioError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

pub struct GpioPin {
    pin_number: u8,
    mode: PinMode,
    state: OutputState,
}

impl GpioPin {
    pub const fn new(pin_number: u8, mode: PinMode) -> Self {
        Self {
            pin_number,
            mode,
            state: OutputState::Low,
        }
    }

    pub fn pin_number(&self) -> u8 {
        self.pin_number
    }

    pub fn mode(&self) -> PinMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: PinMode) {
        self.mode = mode;
    }

    /// Read raw state directly
    pub fn read_state(&self) -> OutputState {
        self.state
    }
}

impl ErrorType for GpioPin {
    type Error = GpioError;
}

impl OutputPin for GpioPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        match self.mode {
            PinMode::OutputPushPull | PinMode::OutputOpenDrain => {
                self.state = OutputState::Low;
                Ok(())
            }
            _ => Err(GpioError::InvalidMode),
        }
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        match self.mode {
            PinMode::OutputPushPull | PinMode::OutputOpenDrain => {
                self.state = OutputState::High;
                Ok(())
            }
            _ => Err(GpioError::InvalidMode),
        }
    }
}

impl StatefulOutputPin for GpioPin {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.state == OutputState::High)
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.state == OutputState::Low)
    }
}

impl InputPin for GpioPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        match self.mode {
            PinMode::InputFloating | PinMode::InputPullUp | PinMode::InputPullDown => {
                Ok(self.state == OutputState::High)
            }
            _ => Err(GpioError::InvalidMode),
        }
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        match self.mode {
            PinMode::InputFloating | PinMode::InputPullUp | PinMode::InputPullDown => {
                Ok(self.state == OutputState::Low)
            }
            _ => Err(GpioError::InvalidMode),
        }
    }
}
