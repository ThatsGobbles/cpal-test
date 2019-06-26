use std::f32::consts::PI;

use cpal::EventLoop;
use cpal::UnknownTypeOutputBuffer;
use cpal::StreamData;

const AMPLITUDE: f32 = 0.25;
const FREQUENCY: f32 = 440.0;

#[derive(Clone, Copy)]
pub enum WaveFunction {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    SineMag,
}

impl WaveFunction {
    pub fn val(&self, sample_clock: u32, sample_rate: u32) -> f32 {
        let f_x = sample_clock as f32 * FREQUENCY / sample_rate as f32;
        AMPLITUDE * match self {
            &WaveFunction::Sine => (2.0 * PI * f_x).sin(),
            &WaveFunction::Square => (-1.0f32).powf((2.0 * f_x).floor()),
            &WaveFunction::Triangle => 1.0 - 4.0 * (0.5 - (f_x + 0.25).fract()).abs(),
            &WaveFunction::Sawtooth => 2.0 * f_x.fract() - 1.0,
            &WaveFunction::SineMag => 2.0 * (PI * f_x).sin().abs() - 1.0,
        }
    }
}

pub struct WaveGen {
    function: WaveFunction,
    sample_rate: u32,
    sample_clock: u32,
}

impl WaveGen {
    pub fn new(function: WaveFunction, sample_rate: u32) -> Self {
        Self {
            function,
            sample_rate,
            sample_clock: 0,
        }
    }

    pub fn step(&mut self) -> f32 {
        let v = self.function.val(self.sample_clock, self.sample_rate);
        self.sample_clock = (self.sample_clock + 1) % self.sample_rate;
        v
    }
}

pub fn run() {
    let device = cpal::default_output_device().unwrap();
    let format = device.default_output_format().unwrap();

    let event_loop = EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let sample_rate = format.sample_rate.0;

    let mut wave_gen = WaveGen::new(WaveFunction::Sine, sample_rate);

    event_loop.run(move |_id, data| {
        match data {
            StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = ((wave_gen.step() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = (wave_gen.step() * std::i16::MAX as f32) as i16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = wave_gen.step();
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            _ => (),
        }
    });
}
