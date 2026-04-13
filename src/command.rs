//! Command parsing and response formatting for USB CDC serial protocol.
//!
//! Commands are text lines terminated by \n or \r. Responses use "=== " prefix.
//! Unsolicited events use "=== EVENT: " prefix.

/// Parsed command from USB serial input.
pub enum Command {
    Boot,
    Ping,
    Id,
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
