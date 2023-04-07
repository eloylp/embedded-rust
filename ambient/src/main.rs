#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;

use hal::{
    clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*, timer::TimerGroup, Delay,
    Rtc, IO,
};


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

    let mut i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio10,
        io.pins.gpio8,
        400u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut delay = Delay::new(&clocks);
    i2c.write(0x70, &[0x35]).ok(); // wake up MSB
    delay.delay_ms(1000u32);

    loop {
        let mut data = [0u8; 24];
        i2c.write_read(0x70, &[0x5c], &mut data).ok();
        println!("{:02x?}", data);
        delay.delay_ms(1000u32)
    }
}
