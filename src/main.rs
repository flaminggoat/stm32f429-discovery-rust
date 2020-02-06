#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
//use cortex_m_semihosting::hio;
//use embedded_graphics::fonts::Font12x16;
use embedded_graphics::image::ImageBmp;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Rectangle};
use embedded_hal::digital::v2::OutputPin;
// use generic_array::{ArrayLength, GenericArray};
use ili9341;
use oorandom;
use panic_semihosting as _;

use crate::hal::{
    prelude::*,
    serial::{self, Serial},
    spi::Spi,
    stm32,
};
use stm32f4xx_hal as hal;

// fn draw_rand_rect<SPI, CS, DC, RESET>(&mut lcd: ili9341::Ili9341<SPI, CS, DC, RESET>) {

// }

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let _start = cortex_m_rt::heap_start() as usize;
        let _size = 1024;

        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(160.mhz()).freeze();

        //let mut stdout = hio::hstdout().unwrap();

        let gpiog = p.GPIOG.split();
        let gpioa = p.GPIOA.split();
        let gpiof = p.GPIOF.split();
        let gpioc = p.GPIOC.split();
        let gpiod = p.GPIOD.split();
        let gpiob = p.GPIOB.split();

        let mut led0 = gpiog.pg13.into_push_pull_output();
        let mut led1 = gpiog.pg14.into_push_pull_output();

        let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

        let en = gpiof.pf10.into_push_pull_output();
        // ------------ SPI INTERFACE SETUP ----------------

        // let spi = Spi::spi5(
        //     p.SPI5,
        //     (
        //         gpiof.pf7.into_alternate_af5(),
        //         hal::spi::NoMiso,
        //         gpiof.pf9.into_alternate_af5(),
        //     ),
        //     ili9341::spi::MODE,
        //     20_000_000.hz(),
        //     clocks,
        // );

        //let cs = gpioc.pc2.into_push_pull_output();
        //let dc = gpiod.pd13.into_push_pull_output();

        //let if_spi = ili9341::spi::SpiInterface::new(spi, cs, dc);

        // ---------- PARALLEL GPIO8 interface setup -----------

        // Set display into 8bit parallel mode
        // Need to bridge jumpers SB23 and SB24 on PCB
        let mut im1 = gpiod.pd4.into_push_pull_output();
        let mut im2 = gpiod.pd5.into_push_pull_output();
        im1.set_low().unwrap();
        im2.set_low().unwrap();

        let csx = gpioc.pc2.into_push_pull_output();
        let wrx = gpiod.pd13.into_push_pull_output();
        let rdx = gpiod.pd12.into_push_pull_output();
        let dcx = gpiof.pf7.into_push_pull_output();

        let mut data_pins: [&mut dyn OutputPin<Error = _>; 8] = [
            &mut gpiod.pd6.into_push_pull_output(),
            &mut gpiog.pg11.into_push_pull_output(),
            &mut gpiog.pg12.into_push_pull_output(),
            &mut gpioa.pa3.into_push_pull_output(),
            &mut gpiob.pb8.into_push_pull_output(),
            &mut gpiob.pb9.into_push_pull_output(),
            &mut gpioa.pa6.into_push_pull_output(),
            &mut gpiog.pg10.into_push_pull_output(),
        ];

        let if_gpio = ili9341::gpio::Gpio8Interface::new(&mut data_pins, csx, wrx, rdx, dcx);

        let mut lcd = ili9341::Ili9341::new(if_gpio, en, &mut delay).unwrap();
        lcd.set_orientation(ili9341::Orientation::Landscape)
            .unwrap();

        let uart_tx_pin = gpioa.pa9.into_alternate_af7();
        let uart_rx_pin = gpioa.pa10.into_alternate_af7();
        let uart = Serial::usart1(
            p.USART1,
            (uart_tx_pin, uart_rx_pin),
            serial::config::Config::default().baudrate(115200.bps()),
            clocks,
        )
        .unwrap();

        let (tx, _rx) = uart.split();
        let mut stdout = tx;

        let temp_cal = hal::signature::VtempCal30::get().read();
        writeln!(stdout, "Temp Cal 30: {}", temp_cal).unwrap();

        writeln!(stdout, "Initialised").unwrap();

        let image = ImageBmp::new(include_bytes!("../cat.bmp"))
            .unwrap()
            .translate(Point::new(30, 30));
        lcd.draw(&image);

        let mut rng = oorandom::Rand32::new(0);
        let mut i = 0;
        let mut x: i32 = 10;
        let mut y: i32 = 10;
        let r = 20;
        let mut xvel = 4;
        let mut yvel = 4;
        loop {
            led0.set_high().unwrap();
            led1.set_high().unwrap();

            let col = Rgb565::new(
                rng.rand_range(0..255) as u8,
                rng.rand_range(0..255) as u8,
                rng.rand_range(0..255) as u8,
            );

            if x >= lcd.width() as i32 {
                xvel = -1 * rng.rand_range(1..10) as i32
            } else if x <= 0 {
                xvel = rng.rand_range(1..10) as i32;
            }
            if y >= lcd.height() as i32 {
                yvel = -1 * rng.rand_range(1..10) as i32;
            } else if y <= 0 {
                yvel = rng.rand_range(1..10) as i32;
            }

            x += xvel;
            y += yvel;

            //let rect = Rectangle::new(Point::new(x, y), Point::new(x+r as i32, y+r as i32)).fill(Some(col));
            // let text = Font12x16::render_str("Hello world")
            //     .style(Style::stroke(Rgb565::RED))
            //     .translate(Point::new(x, y));
            let c = Circle::new(Point::new(x, y), r)
                .stroke(Some(col))
                .stroke_width(5); //.fill(Some(Rgb565::BLUE));
            lcd.draw(c);
            // lcd.draw(text);

            led0.set_low().unwrap();
            led1.set_low().unwrap();
            writeln!(stdout, "Loop: {}", i).unwrap();
            i = i + 1;
        }
    }
    loop {}
}
