use std::fs::File;
use std::sync::Arc;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use rodio::{OutputStream, Sink, source::Source};
use rodio::buffer::SamplesBuffer;

fn main() {

    // Load the SoundFont.
    let mut sf2 = File::open("jeux14.sf2").unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());
    // Create the synthesizer.
    let settings = SynthesizerSettings::new(44100);
    let mut synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();

    // Play some notes (middle C, E, G).
    synthesizer.process_midi_message(1, 0xC0, 124, 0);
    synthesizer.note_on(1, 50, 100);
    //synthesizer.note_on(1, 54, 100);
    //synthesizer.note_on(1, 57, 100);

    // The output buffer (3 seconds).
    let sample_count = (3 * settings.sample_rate) as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    // Render the waveform.
    synthesizer.render(&mut left[..], &mut right[..]);

    // Sound output
    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    let samples =  left
        .into_iter()
        .zip(right.into_iter())
        .flat_map(|(l,r)| vec![l, r])
        .collect::<Vec<f32>>();

    let source = SamplesBuffer::new(2, 44100, samples);


    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source.convert_samples::<f32>());

// The sound plays in a separate thread. This call will block the current thread until the sink
// has finished playing all its queued sounds.
    sink.sleep_until_end();
}