#![allow(unused)]
use rust_esp32_std_demo::bsc;
use bsc::led::*;

use log::*;
use anyhow::bail;
use std::{time::Duration, f32::consts::{PI, E}};

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

const BROWN: RGB8 = RGB8::new(150, 75, 20);
const BLUE: RGB8 = RGB8::new(14, 135, 204);

fn update_buffer(color_buffer0: &mut ColorBuffer, color_buffer1: &mut ColorBuffer, time: std::time::Duration) {
    // flowing effect
    let total_phi = (150.0 * 2.0 * PI) / (5.0 * 8.0) * time.as_secs_f32();
    let phi = total_phi % (PI * 2.0); // [0, 2pi]
    for i in 0..color_buffer0.len() {
        let rho = (i as f32) * 0.5 - phi;
        let r = (BROWN.r as f32 * rho.sin()) as u8;
        let g = (BROWN.g as f32 * rho.sin()) as u8;
        let b = (BROWN.b as f32 * rho.sin()) as u8;
        color_buffer0.set(i, RGB8::new(r, g, b));

        let r = (BLUE.r as f32 * rho.sin()) as u8;
        let g = (BLUE.g as f32 * rho.sin()) as u8;
        let b = (BLUE.b as f32 * rho.sin()) as u8;
        color_buffer1.set(i, RGB8::new(r, g, b));
    }
}

// precise timings are no important for this effect
// just use time since the controller was started

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let start = std::time::Instant::now();

    let mut color_buffer0 = ColorBuffer::new(100 as usize, ColorOrder::GRB);
    color_buffer0.fill(RGB8::new(0, 0, 0));
    let mut color_buffer1 = ColorBuffer::new(100 as usize, ColorOrder::GRB);
    color_buffer1.fill(RGB8::new(0, 0, 0));

    

    let mut led0 = WS2812RMT::new()?;
    let mut led1 = WS2812RMT::new2(9, 1)?;
    loop {
        let now = std::time::Instant::now();
        let time_from_start = now - start;
        update_buffer(&mut color_buffer0, &mut color_buffer1, time_from_start);
        led0.set_pixels(&color_buffer0)?;
        led1.set_pixels(&color_buffer1)?;
        let rest = std::time::Duration::from_millis(33) - now.elapsed();
        std::thread::sleep(rest);
    }
}