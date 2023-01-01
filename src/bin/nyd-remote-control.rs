#![allow(unused)]
use rust_esp32_std_demo::bsc;
use rust_esp32_std_demo::nyd;

use bsc::led::*;

use log::*;
use anyhow::bail;
use std::time::Duration;

use esp_idf_sys as _;

use esp_idf_svc::eventloop::EspSystemEventLoop;

use esp_idf_hal::{
    peripherals::Peripherals,
    peripheral::Peripheral, gpio::Pin,
};

use esp_idf_svc::{
    espnow::{EspNow, SendStatus, PeerInfo},
    wifi::{EspWifi, WifiWait},
};

use embedded_svc::{
    wifi::{Configuration, ClientConfiguration, AccessPointConfiguration, AuthMethod}
};

struct RemoteState {
    sonar: Option<u32>,
    jelly: Option<u32>,
    barrel: Option<u32>,
}


fn main() -> anyhow::Result<()> {
    use bsc::led::RGB8;

    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;

    // Start the LED off yellow
    let mut color_buffer = ColorBuffer::new(42, ColorOrder::GRB);
    color_buffer.fill(RGB8::new(0, 0, 0));
    let mut led = bsc::led::WS2812RMT::new()?;
    led.set_pixels(&color_buffer)?;


    let pins = peripherals.pins;
    let rows = [pins.gpio0.into(), pins.gpio1.into(), pins.gpio2.into(), pins.gpio3.into()];
    let cols = [pins.gpio4.into(), pins.gpio5.into(), pins.gpio6.into(), pins.gpio7.into()];
    let mut keypad = bsc::keypad::Keypad::new(rows, cols)?;
    let mut buffer = Vec::<(u8, u8)>::with_capacity(4);
    
    println!("{:?}", pins.gpio10.pin());

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        keypad.scan(&mut buffer);
        for (mut idx, mut jdx) in buffer.drain(..) {

            let color = match idx {
                0 => RGB8::new(0, 0, 50),
                1 => RGB8::new(0, 50, 0),
                2 => RGB8::new(50, 0, 0),
                _ => RGB8::new(0, 0, 0),
            };

            color_buffer.set(jdx as usize, color);
            led.set_pixels(&color_buffer);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}