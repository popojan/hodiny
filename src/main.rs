#![windows_subsystem = "windows"]

use std::fs::File;
use std::sync::Arc;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use tinyaudio::{OutputDeviceParameters, run_output_device};
use itertools::Itertools;
use midly::{Format, Header, Smf, Timing, Track, TrackEvent, TrackEventKind};
use midly::MetaMessage::{EndOfTrack, Tempo};
use midly::MidiMessage::{NoteOff, NoteOn, ProgramChange};
use midly::num::{u28, u7};

const TICKS_PER_BEAT: u16 = 60;
const MICROSECONDS_PER_BEAT: u32 = 1_000_000;
const QUARTER_NOTE: u7 = u7::new(58);
const HOUR_NOTE: u7 = u7::new(51);
const QUARTER_TICKS: u28 = u28::new(TICKS_PER_BEAT as u32 * 2);
const HOUR_TICKS: u28 = u28::new(TICKS_PER_BEAT as u32 * 2);
const QUARTERS_HOURS_REST_TICKS: u28 = u28::new(TICKS_PER_BEAT as u32);
const QUARTER_VELOCITY: u7 = u7::new(96);
const HOUR_VELOCITY: u7 = u7::new(127);

fn note(track: &mut Track, key: u7, vel: u7, delta: u28, on_not_off: bool) -> u64 {
    track.push(TrackEvent {
        delta,
        kind: TrackEventKind::Midi {
            channel: 1.into(),
            message: if on_not_off {
                NoteOn {key, vel}
            } else {
                NoteOff {key, vel}
            }
        }
    });
    delta.as_int().into()
}

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

    let mut new_midi = Smf::new(Header { format: Format::SingleTrack, timing: Timing::Metrical(TICKS_PER_BEAT.into()) });
    let mut track = Track::new();
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(
            Tempo(
                MICROSECONDS_PER_BEAT.into())),
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
    let mut duration: u64 = 0;
    duration += note(&mut track, QUARTER_NOTE, QUARTER_VELOCITY, u28::new(0), true);
    duration += note(&mut track, QUARTER_NOTE, QUARTER_VELOCITY, QUARTER_TICKS, true);
    duration += note(&mut track, QUARTER_NOTE, QUARTER_VELOCITY, QUARTER_TICKS, true);
    duration += note(&mut track, HOUR_NOTE,    HOUR_VELOCITY,    QUARTER_TICKS + QUARTERS_HOURS_REST_TICKS, true);
    duration += note(&mut track, HOUR_NOTE,    HOUR_VELOCITY,    HOUR_TICKS, true);
    duration += note(&mut track, HOUR_NOTE,    HOUR_VELOCITY,    HOUR_TICKS, true);
    duration += note(&mut track, QUARTER_NOTE, QUARTER_VELOCITY, HOUR_TICKS, false);
    duration += note(&mut track, HOUR_NOTE,    HOUR_VELOCITY,    HOUR_TICKS, false);

    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(EndOfTrack),
    });
    new_midi.tracks.push(track);
    let mut in_memory = Vec::new();
    new_midi.write(&mut in_memory).unwrap();


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

    let estimated_duration_microseconds = duration as f64 / TICKS_PER_BEAT as f64 * MICROSECONDS_PER_BEAT as f64;
    std::thread::sleep(std::time::Duration::from_micros(estimated_duration_microseconds.round()  as u64));

}
