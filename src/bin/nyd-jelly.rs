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

const HOT_PINK: RGB8 = RGB8::new(255, 16, 240);
const PURPLE: RGB8 = RGB8::new(148, 0, 211);

fn update_buffer(color_buffer: &mut ColorBuffer, time: std::time::Duration) {
    // flowing effect
    let total_phi = (150.0 * 2.0 * PI) / (20.0 * 8.0) * time.as_secs_f32();
    let phi = total_phi % (PI * 2.0); // [0, 2pi]
    for i in 0..color_buffer.len() {
        let rho = i as f32 - phi;
        let r = (HOT_PINK.r as f32 * rho.sin() + PURPLE.r as f32 * rho.cos()) as u8;
        let g = (HOT_PINK.g as f32 * rho.sin() + PURPLE.g as f32 * rho.cos()) as u8;
        let b = (HOT_PINK.b as f32 * rho.sin() + PURPLE.b as f32 * rho.cos()) as u8;
        // let s = rho.sin();
        // let c = rho.cos();
        // let s2 = s*s*0.5;
        // let c2 = c*c*0.5;
        // let r = (HOT_PINK.r as f32 * s2 + PURPLE.r as f32 * c2) as u8;
        // let g = (HOT_PINK.g as f32 * s2 + PURPLE.g as f32 * c2) as u8;
        // let b = (HOT_PINK.b as f32 * s2 + PURPLE.b as f32 * c2) as u8;
        color_buffer.set(i, RGB8::new(r, g, b));
    }
}

// precise timings are no important for this effect
// just use time since the controller was started

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let start = std::time::Instant::now();

    let mut color_buffer = ColorBuffer::new(100 as usize, ColorOrder::GRB);
    color_buffer.fill(RGB8::new(0, 0, 0));

    

    let mut led = WS2811RMT::new()?;
    loop {
        let now = std::time::Instant::now();
        let time_from_start = now - start;
        update_buffer(&mut color_buffer, time_from_start);
        led.set_pixels(&color_buffer)?;
        let rest = std::time::Duration::from_millis(33) - now.elapsed();
        std::thread::sleep(rest);
    }
}