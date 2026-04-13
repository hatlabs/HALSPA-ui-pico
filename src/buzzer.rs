//! Piezo buzzer tone pattern control.
//!
//! Patterns are sequences of (frequency_hz, duration_ms) steps.
//! A frequency of 0 means silence. The main loop calls `update()` each cycle.

/// Buzzer tone pattern.
#[derive(Clone, Copy, PartialEq)]
pub enum BuzzerPattern {
    Off,
    Start,  // Short beep
    Pass,   // Rising two-tone
    Fail,   // Descending three-tone
    Estop,  // Rapid beeping
}

impl BuzzerPattern {
    /// Parse a pattern name from command arguments.
    pub fn from_name(name: &[u8]) -> Option<Self> {
        match name {
            b"OFF" => Some(Self::Off),
            b"START" => Some(Self::Start),
            b"PASS" => Some(Self::Pass),
            b"FAIL" => Some(Self::Fail),
            b"ESTOP" => Some(Self::Estop),
            _ => None,
        }
    }
}

/// A single tone step: frequency in Hz (0 = silence) and duration in loop ticks.
struct ToneStep {
    freq_hz: u16,
    duration_ticks: u32,
}

/// Buzzer controller.
pub struct Buzzer {
    pattern: BuzzerPattern,
    step_index: usize,
    ticks_in_step: u32,
    repeating: bool,
}

impl Buzzer {
    pub fn new() -> Self {
        Self {
            pattern: BuzzerPattern::Off,
            step_index: 0,
            ticks_in_step: 0,
            repeating: false,
        }
    }

    /// Set a new pattern. Immediately starts from the beginning.
    pub fn set_pattern(&mut self, pattern: BuzzerPattern) {
        self.pattern = pattern;
        self.step_index = 0;
        self.ticks_in_step = 0;
        self.repeating = matches!(pattern, BuzzerPattern::Estop);
    }

    /// Advance the pattern and return the current frequency in Hz (0 = off).
    /// Call this every loop iteration (~100us).
    pub fn update(&mut self) -> u16 {
        if self.pattern == BuzzerPattern::Off {
            return 0;
        }

        let steps = pattern_steps(self.pattern);
        if steps.is_empty() || self.step_index >= steps.len() {
            if self.repeating {
                self.step_index = 0;
                self.ticks_in_step = 0;
            } else {
                return 0;
            }
        }

        if self.step_index >= steps.len() {
            return 0;
        }

        let step = &steps[self.step_index];
        self.ticks_in_step += 1;

        if self.ticks_in_step >= step.duration_ticks {
            self.step_index += 1;
            self.ticks_in_step = 0;
        }

        step.freq_hz
    }
}

// Duration conversion: 1 tick = ~100us, so 1ms = 10 ticks
const fn ms(milliseconds: u32) -> u32 {
    milliseconds * 10
}

static STEPS_START: [ToneStep; 1] = [
    ToneStep { freq_hz: 1000, duration_ticks: ms(150) },
];

static STEPS_PASS: [ToneStep; 3] = [
    ToneStep { freq_hz: 800, duration_ticks: ms(200) },
    ToneStep { freq_hz: 0, duration_ticks: ms(50) },
    ToneStep { freq_hz: 1200, duration_ticks: ms(300) },
];

static STEPS_FAIL: [ToneStep; 5] = [
    ToneStep { freq_hz: 1000, duration_ticks: ms(200) },
    ToneStep { freq_hz: 0, duration_ticks: ms(50) },
    ToneStep { freq_hz: 700, duration_ticks: ms(200) },
    ToneStep { freq_hz: 0, duration_ticks: ms(50) },
    ToneStep { freq_hz: 400, duration_ticks: ms(200) },
];

static STEPS_ESTOP: [ToneStep; 2] = [
    ToneStep { freq_hz: 2000, duration_ticks: ms(100) },
    ToneStep { freq_hz: 0, duration_ticks: ms(100) },
];

fn pattern_steps(pattern: BuzzerPattern) -> &'static [ToneStep] {
    match pattern {
        BuzzerPattern::Off => &[],
        BuzzerPattern::Start => &STEPS_START,
        BuzzerPattern::Pass => &STEPS_PASS,
        BuzzerPattern::Fail => &STEPS_FAIL,
        BuzzerPattern::Estop => &STEPS_ESTOP,
    }
}
