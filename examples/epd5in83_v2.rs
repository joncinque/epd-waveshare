#![deny(warnings)]

use embedded_graphics::{
    fonts::{Font12x16, Font6x8, Text},
    prelude::*,
    primitives::{Circle, Line},
    style::PrimitiveStyle,
    text_style,
};
use embedded_hal::prelude::*;
use epd_waveshare::{
    color::*,
    epd5in83_v2::{Display5in83, EPD5in83},
    graphics::{Display, DisplayRotation},
    prelude::*,
};
use linux_embedded_hal::{
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
    Delay, Pin, Spidev,
};

// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues

fn main() -> Result<(), std::io::Error> {
    // Configure SPI
    // Settings are taken from
    let mut spi = Spidev::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(spidev::SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs = Pin::new(26); //BCM7 CE0
    cs.export().expect("cs export");
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).expect("CS Direction");
    cs.set_value(1).expect("CS Value set to 1");

    let busy = Pin::new(5); //pin 29
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");
    //busy.set_value(1).expect("busy Value set to 1");

    let dc = Pin::new(6); //pin 31 //bcm6
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    dc.set_value(1).expect("dc Value set to 1");

    let rst = Pin::new(16); //pin 36 //bcm16
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out).expect("rst Direction");
    rst.set_value(1).expect("rst Value set to 1");

    let mut delay = Delay {};

    let mut epd5in83 =
        EPD5in83::new(&mut spi, cs, busy, dc, rst, &mut delay).expect("eink initalize error");

    //println!("Test all the rotations");
    let mut display = Display5in83::default();

    display.set_rotation(DisplayRotation::Rotate0);
    draw_text(&mut display, "Rotate 0!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate90);
    draw_text(&mut display, "Rotate 90!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate180);
    draw_text(&mut display, "Rotate 180!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate270);
    draw_text(&mut display, "Rotate 270!", 5, 50);

    epd5in83.update_frame(&mut spi, &display.buffer())?;
    epd5in83
        .display_frame(&mut spi)
        .expect("display frame new graphics");
    delay.delay_ms(5000u16);

    //println!("Now test new graphics with default rotation and some special stuff:");
    display.clear_buffer(Color::White);

    // draw a analog clock
    let _ = Circle::new(Point::new(64, 64), 40)
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(30, 40))
        .into_styled(PrimitiveStyle::with_stroke(Black, 4))
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(80, 40))
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);

    // draw white on black background
    let _ = Text::new("It's working-WoB!", Point::new(90, 10))
        .into_styled(text_style!(
            font = Font6x8,
            text_color = White,
            background_color = Black
        ))
        .draw(&mut display);

    // use bigger/different font
    let _ = Text::new("It's working-WoB!", Point::new(90, 40))
        .into_styled(text_style!(
            font = Font12x16,
            text_color = White,
            background_color = Black
        ))
        .draw(&mut display);

    // Demonstrating how to use the partial refresh feature of the screen.
    // Real animations can be used.
    //epd5in83
    //    .set_refresh(&mut spi, &mut delay, RefreshLUT::QUICK)
    //    .unwrap();
    //epd5in83.clear_frame(&mut spi).unwrap();

    // a moving `Hello World!`
    let limit = 10;
    for i in 0..limit {
        draw_text(&mut display, "  Hello World! ", 5 + i * 12, 50);

        epd5in83
            .update_and_display_frame(&mut spi, &display.buffer())
            .expect("display frame new graphics");
        delay.delay_ms(1_000u16);
    }

    // Show a spinning bar without any delay between frames. Shows how «fast»
    // the screen can refresh for this kind of change (small single character)
    display.clear_buffer(Color::White);
    epd5in83
        .update_and_display_frame(&mut spi, &display.buffer())
        .unwrap();

    let spinner = ["|", "/", "-", "\\"];
    for i in 0..10 {
        display.clear_buffer(Color::White);
        draw_text(&mut display, spinner[i % spinner.len()], 10, 100);
        epd5in83
            .update_and_display_frame(&mut spi, &display.buffer())
            .unwrap();
    }

    println!("Finished tests - going to sleep");
    epd5in83.sleep(&mut spi)
}

fn draw_text(display: &mut Display5in83, text: &str, x: i32, y: i32) {
    let _ = Text::new(text, Point::new(x, y))
        .into_styled(text_style!(
            font = Font6x8,
            text_color = Black,
            background_color = White
        ))
        .draw(display);
}
