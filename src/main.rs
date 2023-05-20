#![windows_subsystem = "windows"]

mod config;

use std::fs;
use std::fs::File;
use std::sync::Arc;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use tinyaudio::{OutputDeviceParameters, run_output_device};
use itertools::Itertools;
use midly::{Format, Header, Smf, Timing, Track, TrackEvent, TrackEventKind};
use midly::MetaMessage::{EndOfTrack, Tempo};
use midly::MidiMessage::{NoteOff, NoteOn, ProgramChange};
use midly::num::{u28, u7};
use crate::config::Config;
use chrono::{Local, Timelike};

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

    let config_text = fs::read_to_string("hodiny.toml").unwrap();
    let config: Config = toml::from_str(&config_text).unwrap();

    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: 44100,
        channel_sample_count: 4410,
    };
    let now = Local::now();

    let now_hour = now.hour() % 12;
    let now_minute = (now.minute() as f64/ 15.0).floor() as u32;

    let num_hour_strikes = if now_hour == 0 {12} else {now_hour};
    let num_quarter_strikes = if now_minute == 0 {4} else {now_minute};

    let mut left: Vec<f32> = vec![0_f32; params.channel_sample_count];
    let mut right: Vec<f32> = vec![0_f32; params.channel_sample_count];

    let mut sf2 = File::open("jeux14.sf2").unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());

    let mut new_midi = Smf::new(Header {
        format: Format::SingleTrack,
        timing: Timing::Metrical(
            config.tempo.ticks_per_beat.into())
    });

    let mut track = Track::new();
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(
            Tempo(
                config.tempo.microseconds_per_beat.into())),
    });
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel: 1.into(),
            message: ProgramChange {
                program: config.quarter.program.into(),
            }
        }
    });

    let mut duration: u64 = 0;
    let quarter_note = config.quarter.note.into();
    let quarter_velocity = config.quarter.velocity.into();
    let quarter_ticks = config.quarter.delta.into();
    for strike in 0 .. num_quarter_strikes {
        let delta_ticks = if strike == 0 {
            u28::new(0)
        } else {
            quarter_ticks
        };
        duration += note(&mut track, quarter_note, quarter_velocity, delta_ticks, true);
    }

    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel: 1.into(),
            message: ProgramChange {
                program: config.hour.program.into(),
            }
        }
    });
    let hour_note = config.hour.note.into();
    let hour_velocity = config.hour.velocity.into();
    let hour_ticks = config.hour.delta.into();
    let quarters_hours_rest_ticks = config.striking.rest.into();

    for strike in 0 .. num_hour_strikes {
        let delta_ticks = if strike == 0 {
            quarter_ticks + quarters_hours_rest_ticks
        } else {
            hour_ticks
        };
        duration += note(&mut track, hour_note, hour_velocity, delta_ticks, true);
    }
    duration += note(&mut track, quarter_note, quarter_velocity, hour_ticks, false);
    duration += note(&mut track, hour_note, hour_velocity, hour_ticks, false);

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

    let estimated_duration_microseconds = duration as f64 / config.tempo.ticks_per_beat as f64 * config.tempo.microseconds_per_beat as f64;
    std::thread::sleep(std::time::Duration::from_micros(estimated_duration_microseconds.round()  as u64));

}
