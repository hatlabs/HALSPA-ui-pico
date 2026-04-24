//! RGB LED control with animation states.
//!
//! Each LED state defines behavior: solid color, pulsing, or blinking.
//! The main loop calls `update()` every cycle to advance animations.

/// LED animation state.
#[derive(Clone, Copy, PartialEq)]
pub enum LedState {
    Off,
    PulseWhite,
    SolidYellow,
    SolidGreen,
    SolidRed,
    BlinkRed,
}

impl LedState {
    /// Parse a state name from command arguments.
    pub fn from_name(name: &[u8]) -> Option<Self> {
        match name {
            b"OFF" => Some(Self::Off),
            b"PULSE_WHITE" => Some(Self::PulseWhite),
            b"SOLID_YELLOW" => Some(Self::SolidYellow),
            b"SOLID_GREEN" => Some(Self::SolidGreen),
            b"SOLID_RED" => Some(Self::SolidRed),
            b"BLINK_RED" => Some(Self::BlinkRed),
            _ => None,
        }
    }
}

/// RGB LED controller using PWM duty cycles.
pub struct Led {
    state: LedState,
    phase: u32, // Animation phase counter (incremented each update)
}

impl Led {
    pub fn new() -> Self {
        Self {
            state: LedState::Off,
            phase: 0,
        }
    }

    /// Set a new LED state. Resets animation phase.
    pub fn set_state(&mut self, state: LedState) {
        if self.state != state {
            self.state = state;
            self.phase = 0;
        }
    }

    /// Advance animation and return (r, g, b) duty cycle values (0-255).
    pub fn update(&mut self) -> (u8, u8, u8) {
        self.phase = self.phase.wrapping_add(1);

        match self.state {
            LedState::Off => (0, 0, 0),

            LedState::PulseWhite => {
                // Sine-ish pulse using triangle wave
                let cycle = (self.phase / 20) % 512; // ~50ms per step at 100us loop
                let brightness = if cycle < 256 {
                    cycle as u8
                } else {
                    (511 - cycle) as u8
                };
                (brightness, brightness, brightness)
            }

            LedState::SolidYellow => (255, 100, 0),

            LedState::SolidGreen => (0, 255, 0),

            LedState::SolidRed => (255, 0, 0),

            LedState::BlinkRed => {
                // Blink at ~2Hz (250ms on, 250ms off at 100us loop)
                let on = (self.phase / 2500) % 2 == 0;
                if on { (255, 0, 0) } else { (0, 0, 0) }
            }
        }
    }
}
