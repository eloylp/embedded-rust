#![no_std]
#![no_main]

use esp_backtrace as _;

use hal::{
    clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*, timer::TimerGroup, Delay,
    Rtc, IO,
};
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};
use shtcx::{shtc3, PowerMode};

const DISPLAY_I2C_ADDRESS: u8 = 0x27;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    // disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio10,
        io.pins.gpio8,
        400u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let bus = shared_bus::BusManagerSimple::new(i2c);

    let proxy1 = bus.acquire_i2c();
    let proxy2 = bus.acquire_i2c();

    let mut delay = Delay::new(&clocks);
    let mut sht = shtc3(proxy1);

    let mut lcd = HD44780::new_i2c(proxy2, DISPLAY_I2C_ADDRESS, &mut delay).unwrap();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Visible,
            cursor_blink: CursorBlink::On,
        },
        &mut delay,
    )
    .unwrap();

    lcd.write_str("Welcome Rust", &mut delay).unwrap();
    lcd.set_cursor_pos(20, &mut delay).unwrap();
    lcd.write_str("Galicians !", &mut delay).unwrap();

    delay.delay_ms(4000u32);

    // Prepare display for continuous data streaming.
    lcd.clear(&mut delay).unwrap();
    lcd.reset(&mut delay).unwrap();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Invisible,
            cursor_blink: CursorBlink::Off,
        },
        &mut delay,
    )
    .unwrap();

    loop {
        let measure = sht.measure(PowerMode::NormalMode, &mut delay).unwrap();

        // We only reset the cursor here, and rewrite current characters,
        // as all lines have the same, fixed size.
        lcd.reset(&mut delay).unwrap();

        // Show temperature
        lcd.write_str("TempC : ", &mut delay).unwrap();
        let mut buffer = ryu::Buffer::new();
        let temp = buffer.format(measure.temperature.as_degrees_celsius());
        lcd.write_str(temp, &mut delay).unwrap();

        // Move cursor to next line in display:
        lcd.set_cursor_pos(20, &mut delay).unwrap();

        // Show humidity
        lcd.write_str("HR%   : ", &mut delay).unwrap();
        let mut buffer = ryu::Buffer::new();
        let hr = buffer.format(measure.humidity.as_percent());
        lcd.write_str(hr, &mut delay).unwrap();

        delay.delay_ms(1000u32);
    }
}
