//! GPIO pin assignments for the HALSPA UI Pico hardware.
//!
//! Pin assignments are preliminary — update when the actual PCB is designed.

use rp235x_hal::bsp_pins;

bsp_pins!(
    /// On-board LED (directly on Pico 2 board)
    Gpio25 { name: led },
    /// Start button input (active low, internal pull-up)
    Gpio2 { name: button_start },
    /// E-stop button input (active low, internal pull-up)
    Gpio3 { name: button_estop },
);

/// Debounce time in microseconds (50ms)
pub const DEBOUNCE_US: u64 = 50_000;
