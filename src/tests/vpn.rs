use humantime::format_duration;
use lazy_static::lazy_static;
use std::{thread, time::Duration};

use rustmix::{
    vpn::{self, ExpressVPNStatus},
    web::{
        get_public_ip_async,
        reqwest::{build_client, Client},
    },
    Result,
};

lazy_static! {
    static ref WAIT_TIME: Duration = Duration::from_secs(2);
    static ref CLIENT: Client = build_client().build().unwrap();
}

pub async fn test_expressvpn() -> Result<()> {
    println!("\nTesting Express VPN functions...");

    let expressvpn = vpn::ExpressVPN;
    println!("version {}", expressvpn.version()?);

    let mut status = expressvpn.status()?;

    loop {
        match status {
            ExpressVPNStatus::Connected(e) => {
                println!("Connected {:?}! Should disconnect first.", e);
                expressvpn.disconnect()?;
                status = expressvpn.status()?;
            }
            ExpressVPNStatus::Disconnected => {
                println!("Disconnected");
                break;
            }
            ExpressVPNStatus::Error(e) => {
                println!("Error: {}", e);
                return Ok(());
            }
            ExpressVPNStatus::NotActivated => {
                println!("Not Activated");
                return Ok(());
            }
            ExpressVPNStatus::Unknown => {
                println!("Unknown status");
                return Ok(());
            }
        };
    }

    expressvpn.network_lock(true)?;
    print_ip().await;
    println!("connect -> {:?}", expressvpn.connect()?);
    print_ip().await;
    println!(
        "connect to SMART -> {:?}",
        expressvpn.connect_target("smart")?
    );
    print_ip().await;
    println!(
        "connect to United States -> {:?}",
        expressvpn.connect_target("usny")?
    );
    print_ip().await;
    println!(
        "connect to Frankfurt, Germany -> {:?}",
        expressvpn.connect_target("Germany - Frankfurt - 1")?
    );
    print_ip().await;
    println!("disconnect -> {:?}", expressvpn.disconnect()?);
    print_ip().await;
    println!("locations: {:?}", expressvpn.locations()?);
    println!("all locations: {:?}", expressvpn.all_locations()?);

    Ok(())
}

async fn print_ip() {
    let mut tries = 0u8;

    println!("fetching IP address...");

    while tries < 3 {
        match get_public_ip_async(&CLIENT).await {
            Ok(ip) => {
                println!("IP {}", ip);
                break;
            }
            Err(_) => {
                tries += 1;

                if tries < 3u8 {
                    println!(
                        "No connection! Waiting for {}...",
                        format_duration(*WAIT_TIME)
                    );
                    thread::sleep(*WAIT_TIME);
                    continue;
                }

                println!("No connection!");
            }
        };
    }
}
