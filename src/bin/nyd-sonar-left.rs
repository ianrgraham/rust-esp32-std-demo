#![allow(unused)]
use rust_esp32_std_demo::bsc;
use bsc::led::*;

use log::*;
use anyhow::bail;
use std::{time::Duration, f32::consts::PI};

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

// 96.5 inch diameter vertical
// 48 inch half-diameter horizontal

// scan line n on semicircle
// [2, 6, 8, 9, 11, 12, 12, 13, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 14, 14, 13, 12, 12, 11, 9, 8, 6, 2]

// dimensions: 32x32

const ROWS: usize = 32;
const FPS: usize = 30;

#[derive(Copy, Clone)]
enum Side {
    Right,
    Left
}


fn update_buffer(scan_lines: &[u8; ROWS], skip_pixels: &[u8; ROWS], side: Side, color_buffer: &mut ColorBuffer, time: std::time::Duration, state_logic: &mut (usize, bool)) {

    let total_phi = (150.0 * 2.0 * PI) / (60.0 * 8.0) * time.as_secs_f32();
    let mut n = &mut state_logic.0;
    let mut ready = &mut state_logic.1;
    let phi = total_phi % (PI * 2.0); // [0, 2pi]

    let positions = [(-5.0f32, -12.0f32), (-4.0, -9.0), (-3.0f32, -6.0f32), (-2.0f32, -3.0f32), (-1.0f32, -1.0f32)];

    let mut pos = positions[*n % positions.len()];
    let mut raw_angle = (pos.0).atan2(pos.1) + PI; // [0, 2pi]
    let n_plus_pos = positions[(*n + 1) % positions.len()];
    let n_plus_raw_angle = (n_plus_pos.0).atan2(n_plus_pos.1) + PI; // [0, 2pi]
    if *ready {
        if n_plus_raw_angle < phi {
            *n = (*n + 1) % positions.len();
            *ready = false;
            pos = positions[*n % positions.len()];
            raw_angle = (pos.0).atan2(pos.1) + PI; // [0, 2pi]
        }
    }
    else {
        if raw_angle > phi {
            *ready = true;
        }
    }
    let mut pos_angle = (phi - raw_angle) % (2.0 * PI);
    if pos_angle < 0.0 {
        pos_angle += 2.0 * PI;
    }

    let signs: [bool; ROWS] = match side {
        Side::Right => [false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true],
        Side::Left => [true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false]
    };
    let shift = match side {
        Side::Right => 16,
        Side::Left => 0
    };

    let mut idx = 0;
    let mut y = 0;
    for (line, skips, sign) in itertools::izip!(scan_lines, skip_pixels, &signs) {
        let mut x = match (sign, side) {
            (true, Side::Right) => 16,
            (true, Side::Left) => 16 - line,
            (false, Side::Right) => 16 + line,
            (false, Side::Left) => 15
        };
        for _ in 0..*line {
            let xf = (x as f32) - 15.5;
            let yf = (y as f32) - 15.5;
            let orig_angle = xf.atan2(yf) + PI;
            let mut angle = (phi - orig_angle) % (2.0 * PI);
            if angle < 0.0 {
                angle += 2.0 * PI;
            }

            let mut color = RGB8::new(0, 0, 0);
            color.g = color.g.saturating_add(((-angle * 2.0).exp()*100.0) as u8);
            let x2 = xf - pos.0;
            let y2 = yf - pos.1;
            let dist = (x2*x2 + y2*y2).sqrt();
            color.g = color.g.saturating_add(((-dist*dist*dist*dist*0.25 - pos_angle*0.5).exp()*300.0) as u8);

            color_buffer.set(idx, color);
            if *sign {
                x += 1;
            } else {
                x -= 1;
            }
            idx += 1
        }
        idx += *skips as usize;
        y += 1;       
    }
}


fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let start = std::time::Instant::now();

    let scan_lines: [u8; ROWS] = [2, 6, 8, 9, 11, 12, 12, 13, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 14, 14, 13, 12, 12, 11, 9, 8, 6, 2];
    // let skip_pixels: [u8; ROWS] = [0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    // let side = Side::Right;
    let skip_pixels: [u8; ROWS] = [0, 2, 0, 2, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 2, 0, 2, 0, 0];
    let side = Side::Left;

    let as_u16 = |x: &u8| *x as u16;

    let n_pixels = scan_lines.iter().map(as_u16).sum::<u16>() + skip_pixels.iter().map(as_u16).sum::<u16>();
    let mut color_buffer = ColorBuffer::new(n_pixels as usize, ColorOrder::RGB);
    color_buffer.fill(RGB8::new(0, 0, 0));

    let mut state_logic = (0, false);

    let mut led = WS2811RMT::new()?;
    loop {
        let now = std::time::Instant::now();
        let time_from_start = now - start;
        update_buffer(&scan_lines, &skip_pixels, side, &mut color_buffer, time_from_start, &mut state_logic);
        led.set_pixels(&color_buffer)?;
        
        let rest = std::time::Duration::from_millis(33).saturating_sub(now.elapsed());
        // println!("rest: {:?}", now.elapsed());
        std::thread::sleep(rest);
    }
}