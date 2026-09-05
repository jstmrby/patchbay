use std::{f32::consts, thread, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub fn enumerate_devices() {
    let host = cpal::default_host();
    let devices = host.output_devices().expect("unable to fetch devices");
    devices.for_each(|device| {
        println!("{device}");
    });

    let default_output_device = host
        .default_output_device()
        .expect("unable to fetch default output device");
    let default_output_config = default_output_device
        .default_output_config()
        .expect("unable to fetch default output config");

    println!("{:?}", default_output_config)
}

pub fn play_sine_wave() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("unable to fetch default output device");
    let config = device
        .default_output_config()
        .expect("unable to fetch default output config");

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;
    let frequency = 440.0_f32;
    let mut sample_clock = 0f32;

    let stream = device
        .build_output_stream(
            config.config(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // let mut sample_clock = 0f32;
                for frame in data.chunks_mut(channels) {
                    sample_clock = (sample_clock + 1.0) % sample_rate;
                    let value = (sample_clock * frequency * 2.0 * consts::PI / sample_rate).sin();
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            |err| eprintln!("stream error: {err}"),
            None,
        )
        .expect("unable to build output stream");

    stream.play().expect("unable to start stream");
    thread::sleep(Duration::from_secs(2));
}
