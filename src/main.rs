#![no_std]
#![no_main]

mod buzzer;
mod command;
mod led;
mod pins;

use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use embedded_hal::pwm::SetDutyCycle;
use panic_probe as _;

use rp235x_hal as hal;
use hal::clocks::Clock;
use hal::gpio::FunctionPwm;
use hal::reboot::{reboot, RebootArch, RebootKind};
use hal::usb::UsbBus;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

use buzzer::Buzzer;
use command::Command;
use led::Led;

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

/// Button state tracker with debounce.
struct Button {
    was_pressed: bool,
    last_change_us: u64,
}

impl Button {
    fn new() -> Self {
        Self {
            was_pressed: false,
            last_change_us: 0,
        }
    }

    /// Check button state and return true on a new press event (falling edge, debounced).
    fn update(&mut self, is_low: bool, now_us: u64) -> bool {
        if is_low == self.was_pressed {
            return false;
        }
        if now_us.wrapping_sub(self.last_change_us) < pins::DEBOUNCE_US {
            return false;
        }
        if !is_low {
            return false; // Rising edge — use released() instead
        }
        self.was_pressed = true;
        self.last_change_us = now_us;
        true
    }

    /// Check for a release event (rising edge, debounced).
    fn released(&mut self, is_low: bool, now_us: u64) -> bool {
        if is_low || !self.was_pressed {
            return false;
        }
        if now_us.wrapping_sub(self.last_change_us) < pins::DEBOUNCE_US {
            return false;
        }
        self.was_pressed = false;
        self.last_change_us = now_us;
        true
    }
}

#[hal::entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::watchdog::Watchdog::new(pac.WATCHDOG);
    let sio = hal::sio::Sio::new(pac.SIO);

    const XTAL_FREQ_HZ: u32 = 12_000_000u32;
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    let pins = pins::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Button inputs with internal pull-ups (active low)
    let mut button_start = pins.button_start.into_pull_up_input();
    let mut button_estop = pins.button_estop.into_pull_up_input();

    // SPST switch inputs with internal pull-ups (active low = closed)
    let mut switch_1 = pins.switch_1.into_pull_up_input();
    let mut switch_2 = pins.switch_2.into_pull_up_input();

    // PWM setup for RGB LED
    // GPIO6 = PWM3A, GPIO7 = PWM3B, GPIO8 = PWM4A
    let pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    let mut pwm3 = pwm_slices.pwm3;
    pwm3.set_ph_correct();
    pwm3.enable();
    let _led_r_pin = pins.led_r.into_function::<FunctionPwm>();
    let _led_g_pin = pins.led_g.into_function::<FunctionPwm>();

    let mut pwm4 = pwm_slices.pwm4;
    pwm4.set_ph_correct();
    pwm4.enable();
    let _led_b_pin = pins.led_b.into_function::<FunctionPwm>();

    // PWM setup for buzzer (GPIO10 = PWM5A, separate slice from LEDs)
    let mut pwm5 = pwm_slices.pwm5;
    pwm5.set_ph_correct();
    pwm5.enable();
    let _buzzer_pin = pins.buzzer.into_function::<FunctionPwm>();
    let sys_clock_hz = clocks.system_clock.freq().to_Hz();

    // USB CDC setup
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USB,
        pac.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let usb_bus = unsafe {
        USB_BUS = Some(usb_bus);
        USB_BUS.as_ref().unwrap()
    };

    let mut serial = SerialPort::new(usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x2E8A, 0x000A))
        .strings(&[StringDescriptors::default()
            .manufacturer("Hat Labs")
            .product("HALSPA UI Pico")
            .serial_number("HALSPA-UI")])
        .unwrap()
        .device_class(2)
        .build();

    // Wait for USB to enumerate
    for _ in 0..500 {
        usb_dev.poll(&mut [&mut serial]);
        timer.delay_us(1000);
    }

    let _ = serial.write(b"=== INFO: BOOT\n");

    let mut cmd_buf = [0u8; 64];
    let mut cmd_len = 0usize;
    let mut cmd_overflow = false;

    let mut btn_start = Button::new();
    let mut btn_estop = Button::new();
    let mut sw_1 = Button::new();
    let mut sw_2 = Button::new();
    let mut led_ctrl = Led::new();
    let mut buzzer_ctrl = Buzzer::new();

    let mut last_buzzer_freq: u16 = 0;

    // Main loop
    loop {
        usb_dev.poll(&mut [&mut serial]);

        let now_us = timer.get_counter().ticks();

        // Check buttons
        if btn_start.update(button_start.is_low().unwrap_or(false), now_us) {
            let _ = serial.write(b"=== EVENT: BUTTON_START\n");
            usb_dev.poll(&mut [&mut serial]);
        }
        if btn_estop.update(button_estop.is_low().unwrap_or(false), now_us) {
            let _ = serial.write(b"=== EVENT: BUTTON_ESTOP\n");
            usb_dev.poll(&mut [&mut serial]);
        }
        {
            let sw1_low = switch_1.is_low().unwrap_or(false);
            if sw_1.update(sw1_low, now_us) {
                let _ = serial.write(b"=== EVENT: SWITCH_1_CLOSED\n");
                usb_dev.poll(&mut [&mut serial]);
            } else if sw_1.released(sw1_low, now_us) {
                let _ = serial.write(b"=== EVENT: SWITCH_1_OPEN\n");
                usb_dev.poll(&mut [&mut serial]);
            }
        }
        {
            let sw2_low = switch_2.is_low().unwrap_or(false);
            if sw_2.update(sw2_low, now_us) {
                let _ = serial.write(b"=== EVENT: SWITCH_2_CLOSED\n");
                usb_dev.poll(&mut [&mut serial]);
            } else if sw_2.released(sw2_low, now_us) {
                let _ = serial.write(b"=== EVENT: SWITCH_2_OPEN\n");
                usb_dev.poll(&mut [&mut serial]);
            }
        }

        // Update LED animation
        let (r, g, b) = led_ctrl.update();
        let _ = pwm3.channel_a.set_duty_cycle(r as u16 * 257); // Scale 0-255 to 0-65535
        let _ = pwm3.channel_b.set_duty_cycle(g as u16 * 257);
        let _ = pwm4.channel_a.set_duty_cycle(b as u16 * 257);

        // Update buzzer (pwm5, separate from LED PWM)
        let freq = buzzer_ctrl.update();
        if freq != last_buzzer_freq {
            last_buzzer_freq = freq;
            if freq == 0 {
                let _ = pwm5.channel_a.set_duty_cycle(0);
            } else {
                // With phase-correct PWM: f_pwm = f_sys / (2 * TOP)
                let top = sys_clock_hz / (2 * freq as u32);
                pwm5.set_top(top as u16);
                let _ = pwm5.channel_a.set_duty_cycle(top as u16 / 2);
            }
        }

        // Read USB serial data into command buffer
        let mut buf = [0u8; 64];
        if let Ok(count) = serial.read(&mut buf) {
            for &byte in &buf[..count] {
                if byte == b'\n' || byte == b'\r' {
                    if cmd_overflow {
                        let _ = serial.write(b"=== ERROR: COMMAND TOO LONG\n");
                        usb_dev.poll(&mut [&mut serial]);
                    } else if cmd_len > 0 {
                        let cmd = command::parse(&cmd_buf[..cmd_len]);
                        handle_command(
                            cmd, &mut serial, &mut usb_dev,
                            &mut led_ctrl, &mut buzzer_ctrl,
                        );
                    }
                    cmd_len = 0;
                    cmd_overflow = false;
                } else if cmd_len < cmd_buf.len() {
                    cmd_buf[cmd_len] = byte;
                    cmd_len += 1;
                } else {
                    cmd_overflow = true;
                }
            }
        }

        // Brief delay to avoid tight polling
        timer.delay_us(100);
    }
}

fn handle_command<B: usb_device::bus::UsbBus>(
    cmd: Command,
    serial: &mut SerialPort<B>,
    usb_dev: &mut UsbDevice<B>,
    led_ctrl: &mut Led,
    buzzer_ctrl: &mut Buzzer,
) {
    let write_fn =
        |serial: &mut SerialPort<B>, usb_dev: &mut UsbDevice<B>, data: &[u8]| {
            let _ = serial.write(data);
            usb_dev.poll(&mut [serial]);
        };

    match cmd {
        Command::Boot => {
            write_fn(serial, usb_dev, b"=== INFO: REBOOTING TO BOOTLOADER\n");
            for _ in 0..10000 {
                core::hint::spin_loop();
            }
            reboot(
                RebootKind::BootSel {
                    picoboot_disabled: false,
                    msd_disabled: false,
                },
                RebootArch::Normal,
            );
        }

        Command::Ping => {
            write_fn(serial, usb_dev, b"=== OK: PONG\n");
        }

        Command::Id => {
            write_fn(serial, usb_dev, b"=== OK: ID HALSPA-UI\n");
        }

        Command::Led(state) => {
            led_ctrl.set_state(state);
            write_fn(serial, usb_dev, b"=== OK: LED\n");
        }

        Command::Buzzer(pattern) => {
            buzzer_ctrl.set_pattern(pattern);
            write_fn(serial, usb_dev, b"=== OK: BUZZER\n");
        }

        Command::UnknownLedState => {
            write_fn(serial, usb_dev, b"=== ERROR: Unknown LED state\n");
        }

        Command::UnknownBuzzerPattern => {
            write_fn(serial, usb_dev, b"=== ERROR: Unknown buzzer pattern\n");
        }

        Command::Unknown => {
            write_fn(serial, usb_dev, b"=== ERROR: Unknown command\n");
        }
    }
}
