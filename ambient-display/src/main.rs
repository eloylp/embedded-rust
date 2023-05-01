#![no_std]
#![no_main]

use core::fmt::Write;
use esp_backtrace as _;
use esp_println::println;
use heapless::String;

use hal::{
    clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*, timer::TimerGroup, Delay,
    Rtc, IO,
};

use hd44780_driver::{
    display_size::DisplaySize, Cursor, CursorBlink, Display, DisplayMode, HD44780,
};
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
    lcd.reset(&mut delay).unwrap();
    lcd.reset(&mut delay).unwrap();
    //lcd.set_autoscroll(true, &mut delay).unwrap();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Invisible,
            cursor_blink: CursorBlink::Off,
        },
        &mut delay,
    );
    let mut data = String::<32>::new();
    lcd.write_str("Starting ...", &mut delay).unwrap();
    delay.delay_ms(1000u32);
    lcd.reset(&mut delay).unwrap();
    lcd.write_str("Welcome Rust Galicians !", &mut delay)
        .unwrap();
    delay.delay_ms(1000u32);

    let mut message = String::<32>::new();

    loop {
        lcd.reset(&mut delay).unwrap();
        message.clear();

        let measure = sht.measure(PowerMode::NormalMode, &mut delay).unwrap();
        write!(
            message,
            "Temp: {:.2}C HR: {:.2}%",
            measure.temperature.as_degrees_celsius(),
            measure.humidity.as_percent()
        );

        lcd.write_str(message.as_str(), &mut delay).unwrap();
        delay.delay_ms(5000u32);
    }
}
