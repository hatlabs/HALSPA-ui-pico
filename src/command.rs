//! Command parsing and response formatting for USB CDC serial protocol.
//!
//! Commands are text lines terminated by \n or \r. Responses use "=== " prefix.
//! Unsolicited events use "=== EVENT: " prefix.

use crate::buzzer::BuzzerPattern;
use crate::led::LedState;

/// Parsed command from USB serial input.
pub enum Command {
    Boot,
    Ping,
    Id,
    Led(LedState),
    Buzzer(BuzzerPattern),
    UnknownLedState,
    UnknownBuzzerPattern,
    Unknown,
}

/// Parse a command line (without trailing newline) into a Command.
pub fn parse(line: &[u8]) -> Command {
    if line == b"!BOOT" {
        return Command::Boot;
    }
    if line == b"PING" {
        return Command::Ping;
    }
    if line == b"ID" {
        return Command::Id;
    }
    if line.starts_with(b"LED ") {
        let arg = &line[4..];
        return match LedState::from_name(arg) {
            Some(state) => Command::Led(state),
            None => Command::UnknownLedState,
        };
    }
    if line.starts_with(b"BUZZER ") {
        let arg = &line[7..];
        return match BuzzerPattern::from_name(arg) {
            Some(pattern) => Command::Buzzer(pattern),
            None => Command::UnknownBuzzerPattern,
        };
    }
    Command::Unknown
}

/// Write an OK response line.
#[allow(dead_code)]
pub fn respond_ok<F: FnMut(&[u8])>(write_fn: &mut F, msg: &[u8]) {
    write_fn(b"=== OK: ");
    write_fn(msg);
    write_fn(b"\n");
}

/// Write an error response line.
#[allow(dead_code)]
pub fn respond_error<F: FnMut(&[u8])>(write_fn: &mut F, msg: &[u8]) {
    write_fn(b"=== ERROR: ");
    write_fn(msg);
    write_fn(b"\n");
}

/// Write an info response line.
#[allow(dead_code)]
pub fn respond_info<F: FnMut(&[u8])>(write_fn: &mut F, msg: &[u8]) {
    write_fn(b"=== INFO: ");
    write_fn(msg);
    write_fn(b"\n");
}

/// Write an unsolicited event line.
#[allow(dead_code)]
pub fn respond_event<F: FnMut(&[u8])>(write_fn: &mut F, msg: &[u8]) {
    write_fn(b"=== EVENT: ");
    write_fn(msg);
    write_fn(b"\n");
}
