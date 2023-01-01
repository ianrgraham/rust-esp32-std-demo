use log::*;
use anyhow::bail;
use std::time::{Duration, Instant};

use serde::*;

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


// first sonar index is a test currently
const SONAR_ADDRS: [[u8; 6]; 1] = [*b"\x68\x67\x25\xed\x4f\x00"];
const JELLY_ADDRS: [[u8; 6]; 0] = [];
const BARREL_ADDRS: [[u8; 6]; 0] = [];


fn test() {
    let x = b"\x00";
}

// #[derive(Serialize, Deserialize)]
// pub enum Prop {
//     Sonar,
//     Jelly,
//     Barrel,
// }

const MAX_HOPS: usize = 3;

#[derive(Serialize, Deserialize)]
pub enum Message {
    StartProp(u128, usize),
    Sync(u32),
    ResetAll
}


#[allow(unused)]
pub enum Mode {
    STA,
    AP,
    Both
}

pub fn setup_wifi(
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