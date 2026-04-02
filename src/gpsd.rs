#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "class")]
pub enum GpsdMessage {
    VERSION(Version),
    DEVICES(Devices),
    WATCH(Watch),
    TPV(Box<Tpv>),
    SKY(Box<Sky>),
    PPS(Pps),
    GST(Gst),
    TOFF(Toff),
    OSC(Osc),
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub release: Option<String>,
    pub rev: Option<String>,
    pub proto_major: Option<u32>,
    pub proto_minor: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Devices {
    pub devices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    pub path: Option<String>,
    pub driver: Option<String>,
    pub subtype: Option<String>,
    pub subtype1: Option<String>,
    pub activated: Option<String>,
    pub flags: Option<serde_json::Value>,
    pub native: Option<serde_json::Value>,
    pub bps: Option<serde_json::Value>,
    pub parity: Option<String>,
    pub stopbits: Option<serde_json::Value>,
    pub cycle: Option<serde_json::Value>,
    pub mincycle: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Watch {}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct Tpv {
    pub device: Option<String>,
    pub mode: Option<f64>,
    pub status: Option<f64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altHAE: Option<f64>,
    pub altMSL: Option<f64>,
    pub climb: Option<f64>,
    pub speed: Option<f64>,
    pub track: Option<f64>,
    pub magtrack: Option<f64>,
    pub magvar: Option<f64>,
    pub leapseconds: Option<f64>,
    pub ept: Option<f64>,
    pub epx: Option<f64>,
    pub epy: Option<f64>,
    pub epv: Option<f64>,
    pub eps: Option<f64>,
    pub epc: Option<f64>,
    pub epd: Option<f64>,
    pub eph: Option<f64>,
    pub sep: Option<f64>,
    pub geoidSep: Option<f64>,
    pub ecefx: Option<f64>,
    pub ecefy: Option<f64>,
    pub ecefz: Option<f64>,
    pub ecefvx: Option<f64>,
    pub ecefvy: Option<f64>,
    pub ecefvz: Option<f64>,
    pub ecefpAcc: Option<f64>,
    pub ecefvAcc: Option<f64>,
    pub velN: Option<f64>,
    pub velE: Option<f64>,
    pub velD: Option<f64>,
    pub relN: Option<f64>,
    pub relE: Option<f64>,
    pub relD: Option<f64>,
    pub depth: Option<f64>,
    pub dgpsAge: Option<f64>,
    pub dgpsSta: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct Sky {
    pub device: Option<String>,
    pub gdop: Option<f64>,
    pub hdop: Option<f64>,
    pub pdop: Option<f64>,
    pub tdop: Option<f64>,
    pub vdop: Option<f64>,
    pub xdop: Option<f64>,
    pub ydop: Option<f64>,
    pub nSat: Option<f64>,
    pub uSat: Option<f64>,
    pub prRes: Option<f64>,
    pub qual: Option<f64>,
    pub satellites: Option<Vec<Satellite>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct Satellite {
    pub PRN: Option<f64>,
    pub az: Option<f64>,
    pub el: Option<f64>,
    pub ss: Option<f64>,
    pub used: Option<bool>,
    pub gnssid: Option<f64>,
    pub svid: Option<f64>,
    pub sigid: Option<f64>,
    pub freqid: Option<f64>,
    pub health: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Pps {
    pub device: Option<String>,
    pub real_sec: Option<f64>,
    pub real_nsec: Option<f64>,
    pub clock_sec: Option<f64>,
    pub clock_nsec: Option<f64>,
    pub precision: Option<f64>,
    pub shm: Option<String>,
    #[serde(rename = "qErr")]
    pub q_err: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Gst {
    pub device: Option<String>,
    pub rms: Option<f64>,
    pub major: Option<f64>,
    pub minor: Option<f64>,
    pub orient: Option<f64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub alt: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Toff {
    pub device: Option<String>,
    pub real_sec: Option<f64>,
    pub real_nsec: Option<f64>,
    pub clock_sec: Option<f64>,
    pub clock_nsec: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Osc {
    pub device: Option<String>,
    pub running: Option<bool>,
    pub reference: Option<bool>,
    pub disciplined: Option<bool>,
    pub delta: Option<f64>,
}

pub fn parse_message(line: &str) -> Option<GpsdMessage> {
    match serde_json::from_str(line) {
        Ok(msg) => Some(msg),
        Err(e) => {
            tracing::debug!("skipping unparseable gpsd message: {e}");
            None
        }
    }
}
