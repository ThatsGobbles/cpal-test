use cpal::EventLoop;
use cpal::UnknownTypeInputBuffer;
use cpal::StreamData;
use cpal::Sample;
use hound::WavWriter;

pub fn run() {
    // Setup the default input device and stream with the default input format.
    let device = cpal::default_input_device().expect("failed to get default input device");
    let format = device.default_input_format().expect("failed to get default input format");

    println!("Default input device: {}", device.name());
    println!("Default input format: {:?}", format);

    let event_loop = EventLoop::new();
    let stream_id = event_loop.build_input_stream(&device, &format).expect("failed to build input stream");
    event_loop.play_stream(stream_id);

    // The WAV file we're recording to.
    const PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    let spec = wav_spec_from_format(&format);
    let writer = WavWriter::create(PATH, spec).expect("failed to create output wav file");
    let writer = std::sync::Arc::new(std::sync::Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    println!("Begin recording...");
    let recording = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();
    let recording_2 = recording.clone();
    std::thread::spawn(move || {
        event_loop.run(move |_id, data| {
            // If we're done recording, return early.
            if !recording_2.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            // Otherwise write to the wav writer.
            match data {
                StreamData::Input { buffer: UnknownTypeInputBuffer::U16(buffer) } => {
                    if let Ok(mut guard) = writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for sample in buffer.iter() {
                                let sample = Sample::to_i16(sample);
                                writer.write_sample(sample).ok();
                            }
                        }
                    }
                },
                StreamData::Input { buffer: UnknownTypeInputBuffer::I16(buffer) } => {
                    if let Ok(mut guard) = writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for &sample in buffer.iter() {
                                writer.write_sample(sample).ok();
                            }
                        }
                    }
                },
                StreamData::Input { buffer: UnknownTypeInputBuffer::F32(buffer) } => {
                    if let Ok(mut guard) = writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for &sample in buffer.iter() {
                                writer.write_sample(sample).ok();
                            }
                        }
                    }
                },
                _ => (),
            }
        });
    });

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(3));
    recording.store(false, std::sync::atomic::Ordering::Relaxed);
    writer.lock().unwrap().take().unwrap().finalize().unwrap();
    println!("Recording {} complete!", PATH);
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    match format {
        cpal::SampleFormat::U16 => hound::SampleFormat::Int,
        cpal::SampleFormat::I16 => hound::SampleFormat::Int,
        cpal::SampleFormat::F32 => hound::SampleFormat::Float,
    }
}

fn wav_spec_from_format(format: &cpal::Format) -> hound::WavSpec {
    hound::WavSpec {
        channels: format.channels as _,
        sample_rate: format.sample_rate.0 as _,
        bits_per_sample: (format.data_type.sample_size() * 8) as _,
        sample_format: sample_format(format.data_type),
    }
}
