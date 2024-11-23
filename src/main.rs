use clap::Parser;
use notify_rust::Notification;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

/// Get Notifications on critical battery levels
#[derive(Parser, Debug)]
#[command(name = "pwr_notify", about, long_about = None)]
struct Args {
    /// battery name in /sys/class/power_supply
    #[arg(long,short, default_values_t = vec!["BAT1".to_owned()])]
    bats: Vec<String>,

    /// battery threshold in percent all batteries have to be under to trigger notifications on discharge
    #[arg(long, default_value_t = 20)]
    threshold: u8,

    /// notification timeout in seconds, 0 makes it stay until closed
    #[arg(long, default_value_t = 10)]
    timeout: u64,

    /// polling interval in seconds
    #[arg(long, default_value_t = 60*3)]
    polling_intervall: u64,
}

fn main() {
    let args = Args::parse();
    let bats = args.bats;

    loop {
        let mut statuses: Vec<char> = Vec::with_capacity(bats.len());

        for bat in bats.iter() {
            let status = status(bat);

            match status {
                Ok(val) => statuses.push(val),
                Err(e) => notify("Reading Battery Status Failed!", &format!("{e}"), 0),
            };
        }

        let percent = percent_combined(&bats);
        match percent {
            Ok(percent) => {
                let not_charging = !statuses.iter().any(|s| *s == '+');
                let below_threshold = percent < args.threshold;

                if not_charging && below_threshold {
                    notify(
                        "Battery Level Critical!",
                        &format!("{percent}%"),
                        args.timeout,
                    )
                }
            }
            Err(e) => notify("Reading Battery Level Failed!", &format!("{e}"), 0),
        };

        thread::sleep(Duration::from_secs(args.polling_intervall));
    }
}

fn notify(summary: &str, body: &str, timeout: u64) {
    let r = Notification::new()
        .summary(summary)
        .body(body)
        .icon("battery")
        .timeout(Duration::from_secs(timeout))
        .show();

    match r {
        Err(e) => eprintln!("{}", e),
        _ => {}
    }
}

fn read_battery_file(bat: &str, filename: &str) -> Result<String, Box<dyn Error>> {
    let path = PathBuf::from(format!("/sys/class/power_supply/{}/{}", bat, filename));
    Ok(fs::read_to_string(&path)?.trim().to_string())
}

fn read_energy(bat: &str, energy_kind: &str) -> Result<u64, Box<dyn Error>> {
    let content = read_battery_file(bat, energy_kind)?;
    Ok(content.parse()?)
}

fn percent_combined(bats: &[String]) -> Result<u8, Box<dyn Error>> {
    let (mut full_sum, mut now_sum) = (0u64, 0u64);

    for bat in bats {
        let full = read_energy(bat, "energy_full")?;
        let now = read_energy(bat, "energy_now")?;

        full_sum += full;
        now_sum += now;
    }

    if full_sum == 0 {
        Err("Total full energy is zero".into())
    } else {
        Ok(((now_sum as f64 / full_sum as f64) * 100.0) as u8)
    }
}

fn status(bat: &str) -> Result<char, Box<dyn Error>> {
    let content = read_battery_file(bat, "status")?;
    Ok(match content.as_str() {
        "Discharging" => '-',
        "Charging" => '+',
        _ => '?',
    })
}
