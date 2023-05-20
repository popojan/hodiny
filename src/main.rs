#![windows_subsystem = "windows"]

use std::fs::File;
use std::sync::Arc;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use tinyaudio::{OutputDeviceParameters, run_output_device};
use itertools::Itertools;
use midly::{Format, Header, Smf, Timing, Track, TrackEvent, TrackEventKind};
use midly::MetaMessage::{EndOfTrack, Tempo};
use midly::MidiMessage::{NoteOff, NoteOn, ProgramChange};

fn main() {

    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: 44100,
        channel_sample_count: 4410,
    };

    let mut left: Vec<f32> = vec![0_f32; params.channel_sample_count];
    let mut right: Vec<f32> = vec![0_f32; params.channel_sample_count];

    let mut sf2 = File::open("jeux14.sf2").unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());

    let mut new_midi = Smf::new(Header { format: Format::SingleTrack, timing: Timing::Metrical(128.into()) });
    let mut track = Track::new();
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(
            Tempo(
                1000_000.into())),
    });
    track.push(TrackEvent {
       delta: 0.into(),
       kind: TrackEventKind::Midi {
           channel: 1.into(),
           message: ProgramChange {
               program:
               123.into(),
           }
       }
    });
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel: 1.into(),
            message: NoteOn {
                key: 58.into(),
                vel: 96.into(),
            }
        }
    });
    track.push(TrackEvent {
        delta: 128.into(),
        kind: TrackEventKind::Midi {
            channel: 1.into(),
            message: NoteOn {
                key: 51.into(),
                vel: 96.into(),
            }
        }
    });
    track.push(TrackEvent {
        delta: 256.into(),
        kind: TrackEventKind::Midi {
            channel: 1.into(),
            message: NoteOff {
                key: 58.into(),
                vel: 96.into(),
            }
        }
    });
    track.push(TrackEvent {
        delta: 256.into(),
        kind: TrackEventKind::Midi {
            channel: 1.into(),
            message: NoteOff {
                key: 51.into(),
                vel: 96.into(),
            }
        }
    });

    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(EndOfTrack),
    });
    new_midi.tracks.push(track);
    let mut in_memory = Vec::new();
    new_midi.write(&mut in_memory).unwrap();


    //eprintln!("in_memory.len() {}", in_memory.len());

    let mid = MidiFile::new(&mut in_memory.as_slice()).unwrap();

    let midi_file = Arc::new(mid);

    let settings = SynthesizerSettings::new(params.sample_rate as i32);
    let synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();

    let mut sequencer = MidiFileSequencer::new(synthesizer);

    sequencer.play(&midi_file, false);

    let _device = run_output_device(params, {
        move |data| {
            sequencer.render(&mut left[..], &mut right[..]);
            for (i, value) in left.iter().interleave(right.iter()).enumerate() {
                data[i] = *value;
            }
        }
    })
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

}
