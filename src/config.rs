use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub tempo: Tempo,
    pub striking: Striking,
    pub quarter: Strike,
    pub hour: Strike,
}

#[derive(Deserialize)]
pub struct Striking {
    pub kind: u8,
    pub rest: u32,
}

#[derive(Deserialize)]
pub struct Tempo {
    pub ticks_per_beat: u16,
    pub microseconds_per_beat: u32,
}

#[derive(Deserialize)]
pub struct Strike {
    pub program: u8,
    pub note: u8,
    pub delta: u32,
    pub velocity: u8,
}
