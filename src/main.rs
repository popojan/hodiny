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
use std::env;

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

fn simple_or_full(mut track: &mut Track, config: &Config) -> u64 {
    let now = Local::now();
    let now_hour = now.hour() % 12;
    let now_minute = (now.minute() as f64 / 15.0).floor() as u32;

    let num_hour_strikes = if now_hour == 0 {12} else {now_hour};
    let num_quarter_strikes = if now_minute == 0 {4} else {now_minute};

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
    let quarters_hours_rest_ticks = config.striking.rest.into();

    for strike in 0 .. num_quarter_strikes {
        let delta_ticks = if strike == num_quarter_strikes - 1 {
            quarter_ticks + quarters_hours_rest_ticks
        } else {
            quarter_ticks
        };
        duration += note(&mut track, quarter_note, quarter_velocity, u28::new(0), true);
        duration += note(&mut track, quarter_note, quarter_velocity, delta_ticks, false);
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
    let hour_ticks = config.hour.delta.into();
    let hour_note = config.hour.note.into();
    let hour_velocity = config.hour.velocity.into();

    if config.striking.kind > 0 || num_quarter_strikes == 4 {
        for strike in 0..num_hour_strikes {
            let delta_ticks = if strike == num_hour_strikes - 1 {
                hour_ticks + quarters_hours_rest_ticks
            } else {
                hour_ticks
            };
            duration += note(&mut track, hour_note, hour_velocity, u28::new(0), true);
            duration += note(&mut track, hour_note, hour_velocity, delta_ticks, false);
        }
    }
    let extra = hour_ticks.as_int() as u64 * 4;
    duration += extra;

    track.push(TrackEvent {
        delta: u28::new(extra as u32),
        kind: TrackEventKind::Meta(EndOfTrack),
    });

    return duration;
}

fn westminster(mut track: &mut Track, config: &Config) -> u64 {
    let now = Local::now();
    let now_hour = now.hour() % 12;
    let now_minute = (now.minute() as f64 / 15.0).floor() as u32;

    let num_hour_strikes = if now_hour == 0 {12} else {now_hour};
    let num_quarter_strikes = if now_minute == 0 {4} else {now_minute};

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
    /*  https://en.wikipedia.org/wiki/Westminster_Quarters
            G♯4, F♯4, E4, B3	68 66 64 59
            E4, G♯4, F♯4, B3	64 68 66 59
            E4, F♯4, G♯4, E4	64 66 68 64
            G♯4, E4, F♯4, B3	68 64 66 59
            B3, F♯4, G♯4, E4	59 66 68 64
    */
    let changes = vec![
        vec![68, 66, 64, 59],
        vec![64, 68, 66, 59],
        vec![64, 66, 68, 64],
        vec![68, 64, 66, 59],
        vec![59, 66, 68, 64],
    ];
    let quarter_changes = vec![
        vec![1], vec![2, 3], vec![4, 5, 1], vec![2, 3, 4, 5]
    ];
    let mut duration: u64 = 0;
    let quarter_velocity = config.quarter.velocity.into();
    let quarter_ticks: u32 = config.quarter.delta;
    let quarters_hours_rest_ticks: u32 = config.striking.rest.into();

    for (i_change, &change) in quarter_changes[num_quarter_strikes as usize-1].iter().enumerate() {
        for (i_note, &n) in changes[change-1].iter().enumerate() {
            let delta_ticks = if i_note == 3 {
                if i_change == quarter_changes[num_quarter_strikes as usize-1].len() - 1 {
                    (quarter_ticks * 2 + quarters_hours_rest_ticks as u32).into()
                } else {
                    (quarter_ticks * 2).into()
                }
            } else {
                quarter_ticks.into()
            };
            duration += note(&mut track, n.into(), quarter_velocity, u28::new(0), true);
            duration += note(&mut track, n.into(), quarter_velocity, delta_ticks, false);
        }
    }
    let hour_ticks: u32 = config.hour.delta.into();
    if num_quarter_strikes == 4 || config.striking.kind > 2 {
        let hour_note = u7::new(52);
        let hour_velocity = config.hour.velocity.into();

        for strike in 0..num_hour_strikes {
            let delta_ticks = if strike == num_hour_strikes - 1 {
                hour_ticks + quarters_hours_rest_ticks
            } else {
                hour_ticks
            }.into();
            duration += note(&mut track, hour_note, hour_velocity, u28::new(0), true);
            duration += note(&mut track, hour_note, hour_velocity, delta_ticks, false);
        }
    }

    let extra = hour_ticks as u64 * 4;
    duration += extra;

    track.push(TrackEvent {
        delta: u28::new(extra as u32),
        kind: TrackEventKind::Meta(EndOfTrack),
    });
    return duration;
}

fn main() {

    let config_path = if let Ok(Some(config_path)) = env::args().skip(1).at_most_one() {
        config_path
    } else {
        "hodiny.toml".to_string()
    };
    let previous_dir = env::current_dir().ok().unwrap();
    match env::current_exe() {
        Ok(mut exe_path) => {
            exe_path.pop();
            env::set_current_dir(exe_path).ok();
        },
        Err(_e) => return,
    };

    let config_text = fs::read_to_string(config_path).unwrap();
    let config: Config = toml::from_str(&config_text).unwrap();

    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: 44100,
        channel_sample_count: 4410,
    };

    let mut sf2 = File::open(&config.striking.soundfont).unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());

    let mut new_midi = Smf::new(Header {
        format: Format::SingleTrack,
        timing: Timing::Metrical(
            config.tempo.ticks_per_beat.into())
    });

    let mut track = Track::new();

    let duration = if config.striking.kind < 2 {
        simple_or_full(&mut track, &config)
    } else {
        westminster(&mut track, &config)
    };

    new_midi.tracks.push(track);
    let mut in_memory = Vec::new();
    new_midi.write(&mut in_memory).unwrap();

    let mid = MidiFile::new(&mut in_memory.as_slice()).unwrap();

    let midi_file = Arc::new(mid);

    let settings = SynthesizerSettings::new(params.sample_rate as i32);
    let synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();

    let mut sequencer = MidiFileSequencer::new(synthesizer);

    sequencer.play(&midi_file, false);

    let mut left: Vec<f32> = vec![0_f32; params.channel_sample_count];
    let mut right: Vec<f32> = vec![0_f32; params.channel_sample_count];

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

    env::set_current_dir(previous_dir).ok();
}
