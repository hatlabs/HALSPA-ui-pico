---
title: GPIO2 Dead Input Register on Raspberry Pi Pico 2
date: 2026-04-20
category: runtime-errors
module: HALSPA-ui-pico
problem_type: runtime_error
component: tooling
symptoms:
  - "Start button on GPIO2 never generates BUTTON_START events over USB serial"
  - "GPIO2 digital input register always reads high regardless of external state"
  - "E-stop button on GPIO3 works with identical firmware logic"
root_cause: config_error
resolution_type: code_fix
severity: high
related_components:
  - "Raspberry Pi Pico 2 hardware"
  - "HALSPA-runner test infrastructure"
tags:
  - gpio-failure
  - hardware-fault
  - embedded-firmware
  - pin-reassignment
  - pico2
  - rp2350
---

# GPIO2 Dead Input Register on Raspberry Pi Pico 2

## Problem

The start button (wired to GPIO2, active-low with internal pull-up) never generates `=== EVENT: BUTTON_START` events over USB CDC serial. The identical e-stop button on GPIO3 works perfectly. This blocks test automation via HALSPA-runner.

## Symptoms

- Start button press produces no serial event output
- Multimeter confirms button physically pulls GPIO2 low when pressed (wiring correct)
- Firmware reads `button_start.is_low().unwrap_or(false)` — never returns true for GPIO2
- E-stop button on GPIO3 uses identical code path and works reliably
- Behavior persists across firmware rebuilds and reboots

## What Didn't Work

- **Checked firmware logic**: `button_start` and `button_estop` use identical debounce and polling code — GPIO3 works, so firmware is correct
- **Verified USB CDC serial**: Events for GPIO3 arrive fine, ruling out serial issues
- **Toggled pull-up settings and debounce timings**: No change — the input register itself never sees transitions

## Solution

GPIO2 digital input register is dead on this specific Pico 2 unit. Reassigned the start button to a working GPIO:

1. Physically reconnected start button wire from GPIO2 to GPIO11
2. Changed one line in `src/pins.rs`:

```rust
// Before:
Gpio2 { name: button_start },

// After:
Gpio11 { name: button_start },
```

3. No other code changes — firmware references pin by name (`button_start`), not GPIO number
4. Built release UF2, flashed via BOOTSEL (`!BOOT` serial command, copy UF2 to mass storage)
5. Verified: HALSPA-runner receives `BUTTON_START` events

## Why This Works

The `bsp_pins!` macro in `rp235x-hal` maps physical GPIO numbers to named pin handles. All downstream code uses the name, making pin reassignment a single-line change. GPIO11 has a working input register on this unit. The root cause is a silicon-level defect on GPIO2's input register, not a software or configuration issue.

## Prevention

- **When a GPIO input appears dead despite correct wiring**: Test another GPIO with the same signal. If it works, the original pin's input register is likely faulty at the silicon level. Measure with multimeter to rule out electrical issues before prolonged software debugging.
- **New board bring-up**: Include a GPIO input health sweep — toggle each GPIO input with a known signal and verify readback.
- **Firmware pin abstraction**: Use named pin handles (e.g., via `bsp_pins!`) rather than hard-coded GPIO numbers. This makes reassignment a one-line change when hardware issues arise.

## Related Issues

- [hatlabs/HALSPA-ui-pico#3](https://github.com/hatlabs/HALSPA-ui-pico/pull/3) — PR implementing the fix
