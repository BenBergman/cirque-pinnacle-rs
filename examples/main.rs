#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use stm32f1xx_hal::{
    delay::Delay,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    stm32,
};

use cirque::Driver;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let delay = Delay::new(cp.SYST, clocks);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // TODO: add CS and DR pins
    let cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl); // TODO: correct pin and port
    let dr = gpioa.pa3.into_floating_input(&mut gpioa.crl); // TODO: correct pin and port

    let spi = {
        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        Spi::spi1(
            dp.SPI1,
            (sck, miso, mosi),
            &mut afio.mapr,
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            8.mhz(),
            clocks,
            &mut rcc.apb2,
        )
    };

    // TODO: fix these unwrap hacks
    let mut device = Driver::new(spi, cs, dr, delay).unwrap_or_else(|_| panic!("donezo"));

    loop {
        if device.data_ready().unwrap() {
            let pos = device.get_absolute().unwrap_or_else(|_| panic!("donezo"));
            hprintln!("{}\t{}\t{}", pos.x, pos.y, pos.z).unwrap();
        }
    }
}
