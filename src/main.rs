#![windows_subsystem = "windows"]

use std::fs::File;
use std::sync::Arc;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use rodio::{OutputStream, Sink, source::Source};
use rodio::buffer::SamplesBuffer;

fn get_waveform(synthesizer: &mut Synthesizer, channel: i32, note: i32, sample_count: usize, velocity: i32) -> Vec<f32> {
    synthesizer.note_on(channel, note, velocity); //E4 44 B4 51

    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    synthesizer.render(&mut left[..], &mut right[..]);

    left
        .into_iter()
        .zip(right.into_iter())
        .flat_map(|(l,r)| vec![l, r])
        .collect::<Vec<f32>>()
}

fn main() {

    let mut sf2 = File::open("jeux14.sf2").unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());

    let settings = SynthesizerSettings::new(44100);
    let mut synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();

    synthesizer.process_midi_message(1, 0xB0, 0x00, 0); // choose bank
    synthesizer.process_midi_message(1, 0xC0, 123, 0); // choose patch


    let sample_count = (2.5 * settings.sample_rate as f32).round() as usize;
    let quarter_samples = get_waveform(&mut synthesizer, 1, 58, sample_count, 96);
    let hour_samples = get_waveform(&mut synthesizer, 1, 51, sample_count, 96);

    let quarter = || SamplesBuffer::new(2, settings.sample_rate as u32, quarter_samples.clone());
    let hour = || SamplesBuffer::new(2, settings.sample_rate as u32, hour_samples.clone());

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();


    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(quarter().convert_samples::<f32>());
    sink.append(quarter().convert_samples::<f32>());
    sink.append(hour().convert_samples::<f32>());
    sink.append(hour().convert_samples::<f32>());

    sink.sleep_until_end();
}
