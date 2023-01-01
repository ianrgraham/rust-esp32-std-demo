#![allow(unused)]
use rust_esp32_std_demo::bsc;

use log::*;
use anyhow::bail;
use std::time::Duration;

use esp_idf_sys as _;

use esp_idf_svc::eventloop::EspSystemEventLoop;

use esp_idf_hal::{
    peripherals::Peripherals,
    peripheral::Peripheral,
    gpio::{PinDriver, Output, Pull},
};

use esp_idf_svc::{
    espnow::EspNow,
    wifi::{EspWifi, WifiWait},
};

use embedded_svc::{
    wifi::{Configuration, ClientConfiguration, AccessPointConfiguration, AuthMethod}
};

fn main() -> anyhow::Result<()> {
    use bsc::led::RGB8;

    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let mut color_buffer = bsc::led::ColorBuffer::new(400, bsc::led::ColorOrder::RGB);
    // color_buffer.fill(RGB8::new(50, 0, 0));

    // Start the LED off yellow
    let mut led = bsc::led::WS2812RMT::new()?;

    // let mut pin8 = PinDriver::input_output(peripherals.pins.gpio8)?;
    // pin8.set_pull(Pull::Down)?;


    color_buffer.fill(RGB8::new(0, 0, 0));
    led.set_pixels(&color_buffer)?;

    let mut i = 0;
    let mut iup = true;
    let mut j = 33;
    let mut jup = true;
    let mut k = 66;
    let mut kup = true;
    loop {
        // println!("loop");
        if iup {
            i += 1;
            if i == 200 {
                iup = false;
            }
        } else {
            i -= 1;
            if i == 0 {
                iup = true;
            }
        }
        if jup {
            j += 1;
            if j == 22 {
                jup = false;
            }
        } else {
            j -= 1;
            if j == 0 {
                jup = true;
            }
        }
        if kup {
            k += 1;
            if k == 200 {
                kup = false;
            }
        } else {
            k -= 1;
            if k == 0 {
                kup = true;
            }
        }

        let color = RGB8::new(i, i, i);
        color_buffer.fill(color);
        led.set_pixels(&color_buffer)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        println!("{i}");
    }
}