# esp-async-http-server

Cargo.toml
```toml
[package]
name = "http-server"
version = "0.1.0"
authors = ["lz1998 <875543533@qq.com>"]
edition = "2021"
resolver = "2"
rust-version = "1.71"

[profile.release]
opt-level = "s"

[profile.dev]
debug = true    # Symbols are nice and they don't increase the size on Flash
opt-level = "z"

[dependencies]
log = { version = "0.4", default-features = false }
esp-idf-svc = { version = "0.47.3", default-features = false, features = ["alloc", "panic_handler", "alloc_handler", "libstart"] }
embassy-futures = "0.1"
esp-idf-sys = { version = "0.33", default-features = false }
esp-idf-hal = { version = "0.42", default-features = false, features = ["nightly"] }
esp-async-tcp = { git = "https://github.com/lz1998/esp-async-tcp.git", branch = "main" }
esp-async-http-server = { git = "https://github.com/lz1998/esp-async-http-server.git", branch = "main" }


[build-dependencies]
embuild = "0.31.3"
```

main.rs
```rust
#![feature(ip_in_core)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use embassy_futures::select::select;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, AuthMethod, EspWifi};
use esp_idf_sys::EspError;
use esp_async_http_server::Response;

#[no_mangle]
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let res = esp_idf_hal::task::block_on(init());
    log::info!("{res:?}");
}

async fn init() -> Result<(), EspError> {
    let peripherals = Peripherals::take()?;

    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
        timer_service,
    )?;

    let wifi_configuration =
        esp_idf_svc::wifi::Configuration::Mixed(esp_idf_svc::wifi::ClientConfiguration {
            ssid: "XXX".into(),
            auth_method: AuthMethod::WPA2Personal,
            password: "XXX".into(),
            ..Default::default()
        }, esp_idf_svc::wifi::AccessPointConfiguration {
            ssid: "XXXX".into(),
            auth_method: AuthMethod::None,
            password: "".into(),
            channel: 1,
            max_connections: 255,
            ..Default::default()
        });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start().await?;
    log::info!("Wifi started");

    wifi.connect().await?;
    log::info!("Wifi connected");

    wifi.wait_netif_up().await?;
    log::info!("Wifi netif up");
    log::info!("{:?}", wifi.wifi().ap_netif().get_ip_info());

    let mut timer = esp_idf_hal::timer::TimerDriver::new(peripherals.timer00, &esp_idf_hal::timer::TimerConfig::new())?;
    timer.delay(timer.tick_hz()).await.unwrap();
    select(start_http_server(), timer_loop(&mut timer)).await;
    Ok(())
}

async fn start_http_server() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    let listener = esp_async_tcp::TcpListener::bind(&addr).unwrap();
    loop {
        let (mut stream, addr) = listener.accept().await.unwrap();
        if let Err(err) = esp_async_http_server::process_req(&mut stream, addr, handler.clone()).await {
            log::error!("failed to process req: {err:?}");
        }
    }
}

async fn timer_loop(timer: &mut esp_idf_hal::timer::TimerDriver<'_>) {
    let one_ms = timer.tick_hz() / 1000;
    loop {
        timer.delay(one_ms).await.unwrap();
    }
}

async fn router(addr: SocketAddr, req: esp_async_http_server::Request<'_, '_>, body: &[u8]) -> Response {
    return match req.path {
        None => "400".into(),
        Some(p) if p.starts_with("/ping") => ping(addr, req, body).await.into(),
        Some(p) if p.starts_with("/pong") => pong(addr, req, body).await.into(),
        Some(_) => "404".into()
    }
}

async fn ping(addr: SocketAddr, req: esp_async_http_server::Request<'_, '_>, body: &[u8]) -> &'static str {
    log::info!("ping");
    log::info!("REQ_ADDR: {addr}");
    log::info!("REQ_PATH: {:?}",req.path);
    log::info!("REQ_VERSION: {:?}",req.version);
    log::info!("REQ_HEADERS: {:?}",req.headers);
    log::info!("REQ_BODY: {}",String::from_utf8_lossy(body));
    "ping"
}

async fn pong(addr: SocketAddr, req: esp_async_http_server::Request<'_, '_>, body: &[u8]) -> &'static str {
    log::info!("pong");
    log::info!("REQ_ADDR: {addr}");
    log::info!("REQ_PATH: {:?}",req.path);
    log::info!("REQ_VERSION: {:?}",req.version);
    log::info!("REQ_HEADERS: {:?}",req.headers);
    log::info!("REQ_BODY: {}",String::from_utf8_lossy(body));
    "pong"
}
```
