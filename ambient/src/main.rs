#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;

use hal::{
    clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*, timer::TimerGroup, Delay,
    Rtc, IO,
};
use shtcx::{shtc3, PowerMode};

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

    let mut delay = Delay::new(&clocks);
    let mut sht = shtc3(i2c);

    loop {
        let measure = sht.measure(PowerMode::NormalMode, &mut delay).unwrap();
        println!(
            "Temperature: {} C",
            measure.temperature.as_degrees_celsius()
        );
        println!("Humidity: {} %RH", measure.humidity.as_percent());
        delay.delay_ms(5000u32)
    }
}
