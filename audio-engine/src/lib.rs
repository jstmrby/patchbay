use std::{f32::consts, fs::File, thread, time::Duration};

use cpal::{
    SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use symphonia::{
    core::{
        codecs::audio::AudioDecoderOptions,
        errors::Error,
        formats::{FormatOptions, TrackType, probe::Hint},
        io::MediaSourceStream,
        meta::MetadataOptions,
        units::Timestamp,
    },
    default,
};

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

pub fn parse_wav(path: &str) {
    let file = File::open(path).expect("unable to open file");
    // Create the media source stream.
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Create a probe hint using the file's extension. [Optional]
    let mut hint = Hint::new();
    hint.with_extension("wav");

    // Use the default options for metadata and format readers.
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    // Probe the media source.
    let mut format = default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .expect("unsupported format");

    // Find the first audio track with a known (decodeable) codec.
    let track = format
        .default_track(TrackType::Audio)
        .expect("no audio track");

    let time_base = track.time_base.expect("unable to get time_base");
    let duration = track.duration.expect("duration is not specified");
    let time = time_base
        .calc_time(Timestamp::try_from(duration.get()).expect("duration too long"))
        .expect("time could not be converted");
    println!("Duration: {:02}:{:02}", time.as_mins(), time.as_secs() % 60);

    // Use the default options for the decoder.
    let dec_opts: AudioDecoderOptions = Default::default();

    // Create a decoder for the track.
    let mut decoder = default::get_codecs()
        .make_audio_decoder(
            track
                .codec_params
                .as_ref()
                .expect("codec parameters missing")
                .audio()
                .unwrap(),
            &dec_opts,
        )
        .expect("unsupported codec");

    // Store the track identifier, it will be used to filter packets.
    let track_id = track.id;

    let cp = decoder.codec_params();
    let sample_rate = cp.sample_rate.expect("sample rate could not be determined");
    let channel_count = cp.channels.as_ref().expect("no channels found").count();
    let mut sample_data: Vec<f32> = Vec::new();

    // The decode loop.
    loop {
        // Get the next packet from the media format.
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => {
                // Reached the end of the stream.
                break;
            }
            Err(Error::ResetRequired) => {
                // The track list has been changed. Re-examine it and create a new set of decoders,
                // then restart the decode loop. This is an advanced feature and it is not
                // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                // for chained OGG physical streams.
                unimplemented!();
            }
            Err(err) => {
                // A unrecoverable error occurred, halt decoding.
                panic!("{}", err);
            }
        };

        // Consume any new metadata that has been read since the last packet.
        while !format.metadata().is_latest() {
            // Pop the old head of the metadata queue.
            format.metadata().pop();

            // Consume the new metadata at the head of the metadata queue.
        }

        // If the packet does not belong to the selected track, skip over it.
        if packet.track_id != track_id {
            continue;
        }

        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Consume the decoded audio samples (see below).
                let mut decoded_samples: Vec<f32> = Vec::new();
                decoded.copy_to_vec_interleaved(&mut decoded_samples);
                sample_data.append(&mut decoded_samples);
            }
            Err(Error::IoError(_)) => {
                // The packet failed to decode due to an IO error, skip the packet.
                continue;
            }
            Err(Error::DecodeError(_)) => {
                // The packet failed to decode due to invalid data, skip the packet.
                continue;
            }
            Err(err) => {
                // An unrecoverable error occurred, halt decoding.
                panic!("{}", err);
            }
        }
    }

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("unable to fetch default output device");

    let config = StreamConfig::from(SupportedStreamConfig::new(
        channel_count.try_into().expect("channel count too large"),
        sample_rate,
        SupportedBufferSize::default(),
        SampleFormat::F32,
    ));

    let mut sample_iterator = sample_data.into_iter();
    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for d in data.iter_mut() {
                    *d = sample_iterator.next().unwrap_or(0.0)
                }
            },
            |err| eprintln!("stream error: {err}"),
            None,
        )
        .expect("unable to build output stream");

    stream.play().expect("unable to start stream");
    thread::sleep(Duration::from_secs(
        (time.as_secs() + 1)
            .try_into()
            .expect("duration time could not be converted"),
    ));
}
