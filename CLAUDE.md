# HALSPA UI Pico

Rust firmware for the HALSPA test jig UI controls (RGB LED button, e-stop button, buzzer) on RPi Pico 2.

## Build

```bash
./run install    # First time: install toolchain + dependencies
./run build      # Debug build
./run build-uf2  # Release build, produces artifacts/halspa-ui-pico.uf2
./run flash      # Flash via probe-rs (requires debug probe)
```

## Flashing via BOOTSEL

When no debug probe is available, flash via USB mass storage:

1. Send `!BOOT` over USB serial — device reboots into BOOTSEL mode
2. Wait for USB mass storage device to appear (e.g. `/dev/sda1` on Pi, or `RPI-RP2` volume on macOS)
3. Copy `artifacts/halspa-ui-pico.uf2` to the mounted volume
4. Device reboots automatically after copy completes

On Pi:
```bash
# Send !BOOT (assumes device is /dev/ttyACM0)
echo '!BOOT' > /dev/ttyACM0
# Wait for mass storage, then copy
sudo mount /dev/sda1 /mnt
sudo cp artifacts/halspa-ui-pico.uf2 /mnt/
sudo sync
sudo umount /mnt
```

On macOS, the volume mounts automatically as `RPI-RP2` — drag-and-drop or:
```bash
cp artifacts/halspa-ui-pico.uf2 /Volumes/RPI-RP2/
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

Pin assignments in `src/pins.rs`. GPIO11 (start button) was reassigned from GPIO2 due to a hardware fault on this Pico 2 unit (see `GPIO2_ISSUE.md`).

## Architecture

- `src/main.rs` — USB CDC setup, main event loop, button polling, command dispatch
- `src/command.rs` — Command parsing and response formatting
- `src/pins.rs` — Pin assignments via `bsp_pins!` macro
