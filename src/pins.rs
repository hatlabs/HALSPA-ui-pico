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
    /// SPST switch 1 input (active low, internal pull-up)
    Gpio4 { name: switch_1 },
    /// SPST switch 2 input (active low, internal pull-up)
    Gpio5 { name: switch_2 },
    /// RGB LED - Red channel (PWM)
    Gpio6 { name: led_r },
    /// RGB LED - Green channel (PWM)
    Gpio7 { name: led_g },
    /// RGB LED - Blue channel (PWM)
    Gpio8 { name: led_b },
    /// Piezo buzzer (PWM) — on a separate PWM slice from LEDs
    Gpio10 { name: buzzer },
);

/// Debounce time in microseconds (50ms)
pub const DEBOUNCE_US: u64 = 50_000;
