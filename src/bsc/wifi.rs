use anyhow::bail;
use embedded_svc::wifi::{
    ClientConfiguration, Configuration, AccessPointConfiguration, Wifi
};
use esp_idf_svc::{
    netif::{EspNetif, EspNetifWait}, eventloop::EspSystemEventLoop, wifi::{EspWifi, WifiWait},
};
use esp_idf_hal::peripheral;
use log::info;
use std::time::Duration;

pub fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    ssid: &str,
    psk: &str
) -> anyhow::Result<Box<EspWifi<'static>>> {
    use std::net::Ipv4Addr;

    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: ssid.into(),
            password: psk.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    wifi.start()?;

    info!("Starting wifi...");

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    info!("Connecting wifi...");

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    ping(ip_info.subnet.gateway)?;

    Ok(wifi)
}

pub fn wifi_ap(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    ssid: &str,
    psk: &str
) -> anyhow::Result<Box<EspWifi<'static>>> {

    use std::net::Ipv4Addr;

    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);
    let channel = 0;

    info!("Wifi created, about to scan");

    wifi.set_configuration(&Configuration::AccessPoint(
        AccessPointConfiguration {
            ssid: ssid.into(),
            password: psk.into(),
            channel: channel,
            auth_method: embedded_svc::wifi::AuthMethod::WPAWPA2Personal,
            ..Default::default()
        }
    ))?;

    wifi.start()?;

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    Ok(wifi)
}

pub fn wifi_client(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    ssid: &str,
    psk: &str
) -> anyhow::Result<Box<EspWifi<'static>>> {

    use std::net::Ipv4Addr;

    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Client(
        ClientConfiguration {
            ssid: ssid.into(),
            password: psk.into(),
            channel,
            ..Default::default()
        }
    ))?;

    wifi.start()?;

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(wifi)
}

fn ping(ip: embedded_svc::ipv4::Ipv4Addr) -> anyhow::Result<()> {
    info!("About to do some pings for {:?}", ip);

    let ping_summary = esp_idf_svc::ping::EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!("Pinging IP {} resulted in timeouts", ip);
    }

    info!("Pinging done");

    Ok(())
}