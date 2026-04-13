# HALSPA UI Pico

Rust firmware for the HALSPA test jig UI controls (RGB LED button, e-stop button, buzzer) on RPi Pico 2.

## Build

```bash
./run install    # First time: install toolchain + dependencies
./run build      # Debug build
./run build-uf2  # Release build, produces artifacts/halspa-ui-pico.uf2
./run flash      # Flash via probe-rs
```

## USB CDC Protocol

Commands are newline-terminated text. Responses use `=== OK:`, `=== ERROR:`, `=== INFO:` prefixes. Unsolicited button events use `=== EVENT:` prefix.

### Commands

| Command | Response |
|---------|----------|
| `PING`  | `=== OK: PONG` |
| `ID`    | `=== OK: ID HALSPA-UI` |
| `!BOOT` | Reboots to BOOTSEL mode |

### Events (unsolicited)

| Event | Meaning |
|-------|---------|
| `=== EVENT: BUTTON_START` | Start button pressed |
| `=== EVENT: BUTTON_ESTOP` | E-stop button pressed |

## Pin Assignments

Pin assignments in `src/pins.rs` are preliminary. Update when the PCB design is finalized.

## Architecture

- `src/main.rs` — USB CDC setup, main event loop, button polling, command dispatch
- `src/command.rs` — Command parsing and response formatting
- `src/pins.rs` — Pin assignments via `bsp_pins!` macro
