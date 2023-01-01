#![allow(unused)]
use rust_esp32_std_demo::bsc;

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

#[allow(unused)]
enum Mode {
    STA,
    AP,
    Both
}

fn setup_wifi(
    modem: impl Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    mode: Mode
) -> anyhow::Result<Box<EspWifi<'static>>> {
    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

    match mode {
        Mode::STA => wifi.set_configuration(&Configuration::Client(
            ClientConfiguration{
                auth_method: AuthMethod::None,
                ..Default::default()
            }
        ))?,
        Mode::AP => wifi.set_configuration(&Configuration::AccessPoint(
            AccessPointConfiguration{
                auth_method: AuthMethod::None,
                ..Default::default()
            }
        ))?,
        Mode::Both => wifi.set_configuration(&Configuration::Mixed(
            ClientConfiguration{
                auth_method: AuthMethod::None,
                ..Default::default()
            },
            AccessPointConfiguration{
                auth_method: AuthMethod::None,
                ..Default::default()
            })
        )?,
    }

    println!("MACs: {:#04x?} {:#04x?}", wifi.sta_netif().get_mac()?, wifi.ap_netif().get_mac()?);

    info!("WiFi configured");

    wifi.start()?;

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    Ok(wifi)
}

fn main() -> anyhow::Result<()> {
    use bsc::led::RGB8;

    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;

    info!("Starting wifi...");

    // Start the LED off yellow
    let mut led = bsc::led::WS2812RMT::new()?;
    let mut color_buffer = bsc::led::ColorBuffer::new(42, bsc::led::ColorOrder::GRB);
    color_buffer.fill(RGB8::new(0, 0, 0));
    led.set_pixels(&color_buffer)?;

    let mut _wifi = setup_wifi(peripherals.modem, sysloop.clone(), Mode::STA)?;

    println!("Setting up ESPNow...");

    let espnow = EspNow::take()?;

    espnow.register_send_cb(|data, status| {
        // println!("Sending data: {:?}\n...", data);
        // match status {
        //     SendStatus::SUCCESS => println!("Message sent successfully"),
        //     SendStatus::FAIL => println!("Error sending message")
        // }
    })?;

    let n_peers = espnow.get_peers_number()?;
    println!("Number of peers: {:?}", n_peers);

    // receiver 68:67:25:ed:26:68
    // sender   58:cf:79:e9:43:5c
    let address = [0x68, 0x67, 0x25, 0xed, 0x26, 0x68];
    // let address = [0x58, 0xcf, 0x79, 0xe9, 0x43, 0x5c];
    // let peer_info = espnow.get_peer(address)?;
    let peer_info = PeerInfo {
        peer_addr: address,
        channel: 0,
        encrypt: false,
        ifidx: 0,
        ..Default::default()
    };
    println!("Got peer info: {:?}", peer_info);
    espnow.add_peer(peer_info)?;
    println!("Added peer");
    let n_peers = espnow.get_peers_number()?;
    println!("Number of peers: {:?}", n_peers);


    // loop {
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     led.set_pixel(RGB8::new(0, 50, 0))?;
    //     // println!("Sending data: {:?}\n...", &[0, 50, 0]);
    //     espnow.send(address, &[0, 50, 0])?;

    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     led.set_pixel(RGB8::new(0, 50, 50))?;
    //     espnow.send(address, &[0, 50, 50])?;

    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     led.set_pixel(RGB8::new(0, 0, 50))?;
    //     espnow.send(address, &[0, 0, 50])?;
    // }

    led.set_pixel(RGB8::new(0, 50, 0))?;
    // let mut idx = 0;
    // let mut jdx = 0;
    // loop {
    //     std::thread::sleep(std::time::Duration::from_secs(1));

    //     idx = (unsafe{esp_idf_sys::esp_random()}%4) as u8;
    //     jdx = (unsafe{esp_idf_sys::esp_random()}%4) as u8;
    //     let color = match idx {
    //         0 => RGB8::new(0, 0, 50),
    //         1 => RGB8::new(0, 50, 0),
    //         2 => RGB8::new(50, 0, 0),
    //         _ => RGB8::new(0, 0, 0),
    //     };

    //     led.set_pixel(color)?;
    //     espnow.send(address, &[idx, jdx])?;
    // }

    let pins = peripherals.pins;
    let rows = [pins.gpio0.into(), pins.gpio1.into(), pins.gpio2.into(), pins.gpio3.into()];
    let cols = [pins.gpio4.into(), pins.gpio5.into(), pins.gpio6.into(), pins.gpio7.into()];
    let mut keypad = bsc::keypad::Keypad::new(rows, cols)?;
    let mut buffer = Vec::<(u8, u8)>::with_capacity(4);
    
    println!("{:?}", pins.gpio10.pin());

    loop {
        std::thread::sleep(std::time::Duration::from_millis(16));
        keypad.scan(&mut buffer);
        // println!("Buffer len: {}", buffer.len());
        for (idx, jdx) in buffer.drain(..) {
            let color = match idx {
                0 => RGB8::new(0, 0, 50),
                1 => RGB8::new(0, 50, 0),
                2 => RGB8::new(50, 0, 0),
                _ => RGB8::new(0, 0, 0),
            };
            color_buffer.set(jdx as usize, color);
            led.set_pixels(&color_buffer)?;
            
            espnow.send(address, &[idx, jdx]);
        }
    }
}