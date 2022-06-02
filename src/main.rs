extern crate pulsectl;

use evdev::{Device, InputEventKind};

use libpulse_binding::volume::ChannelVolumes;
use libpulse_binding::volume::Volume;
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;

fn main() {
    let mut handler = SinkController::create().unwrap();

    let mut d = Device::open("/dev/input/event29").unwrap();
    println!("{}", d);
    println!("Events:");
    let mut last_value = None;
    loop {
        for ev in d.fetch_events().unwrap() {
            match ev.kind() {
                InputEventKind::AbsAxis(_) => {
                    if let Some(lvalue) = last_value {
                        if ev.value() != lvalue {
                            let calibrated_value: u32 = ((ev.value() as f32 + 127.0) / (127.0 * 2.0) * 32860.0 * 2.0).ceil() as u32;
                            println!("Setting volume at: {}", calibrated_value);
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
}

fn update_volume(handler: &mut SinkController, volume: u32) {
    let device = handler.get_default_device().unwrap();
    let channel_number = device.channel_map.len();
    let mut channel_volumes = ChannelVolumes::default();
    channel_volumes.set(channel_number, Volume(volume));
    handler.set_device_volume_by_index(device.index, &channel_volumes);
}

