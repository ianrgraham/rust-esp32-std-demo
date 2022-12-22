#![allow(unused)]
use rust_esp32_std_demo::bsc;

use log::*;
use anyhow::bail;
use std::time::Duration;

use esp_idf_sys as _;

use esp_idf_svc::eventloop::EspSystemEventLoop;

use esp_idf_hal::{
    peripherals::Peripherals,
    peripheral::Peripheral
};

use esp_idf_svc::{
    espnow::EspNow,
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

    info!("WiFi configured as STA");

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

    let mut color_buffer = bsc::led::ColorBuffer::new(4);
    color_buffer.fill(RGB8::new(50, 0, 0));

    // Start the LED off yellow
    let mut led = bsc::led::WS2812RMT::new()?;
    led.set_pixels(&color_buffer)?;

    let mut _wifi = setup_wifi(peripherals.modem, sysloop.clone(), Mode::STA)?;

    println!("Setting up ESPNow...");

    let espnow = EspNow::take()?;

    // espnow.register_recv_cb(move |mac, data| {
    //     // println!("Received data: {:?} from {:#04x?}", data, mac);
    //     let color = RGB8::new(data[0], data[1], data[2]);
    //     color_buffer.fill(color);
    //     led.set_pixels(&color_buffer).unwrap();
    // })?;

    espnow.register_recv_cb(move |_mac, data| {
        // println!("Received data: {:?} from {:#04x?}", data, mac);
        let color = match data[0] {
            0 => RGB8::new(0, 0, 50),
            1 => RGB8::new(0, 50, 0),
            2 => RGB8::new(50, 0, 0),
            _ => RGB8::new(0, 0, 0),
        };
        color_buffer.set(data[1] as usize, color);
        led.set_pixels(&color_buffer).unwrap();
    })?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}