use cpal::traits::{DeviceTrait, HostTrait};

pub fn enumerate_devices() {
    let host = cpal::default_host();
    let devices = host.output_devices().expect("unable to fetch devices");
    devices.for_each(|device| {
        println!("{device}");
    });

    let default_output_device = host
        .default_output_device()
        .expect("unable to fetch default output device");
    let output_config = default_output_device
        .default_output_config()
        .expect("unable to fetch default output config");
    println!("{:?}", output_config)
}
