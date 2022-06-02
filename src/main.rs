extern crate pulsectl;

use std::thread;
use std::time::Duration;

use evdev::{Device, InputEventKind};

use libpulse_binding::volume::Volume;
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Signaling server address
    #[clap(long)]
    event_path: String,
}

fn main() {
    let args = Args::parse();

    println!("Connecting to pulseaudio...");
    let mut handler = match SinkController::create() {
        Ok(h) => {
            println!("Successfully connected to pulseaudio!");
            h
        },
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let device_path = args.event_path;

    let mut last_value = None;
    loop {
        println!("Connecting to control device: {}", device_path);
        match Device::open(device_path.clone()) {
            Ok(mut d) => {
                println!("Successfully connected to control device!");
                loop {
                    match d.fetch_events() {
                        Ok(events) => {
                            for ev in events {
                                match ev.kind() {
                                    InputEventKind::AbsAxis(_) => {
                                        if let Some(lvalue) = last_value {
                                            if ev.value() != lvalue {
                                                let calibrated_value: u32 = ((ev.value() as f32 + 127.0) / 254.0 * 65720.0).ceil() as u32;
                                                update_volume(&mut handler, calibrated_value);
                                                last_value = Some(ev.value());
                                            }
                                        } else {
                                                last_value = Some(ev.value());
                                            }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("{}", err);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn update_volume(handler: &mut SinkController, volume: u32) {
    // You better have a default device
    let device = handler.get_default_device().unwrap();
    let channel_number = device.channel_map.len();
    let mut channel_volumes = device.volume;
    channel_volumes.set(channel_number, Volume(volume));
    handler.set_device_volume_by_index(device.index, &channel_volumes);
}

