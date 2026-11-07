//! Wire protocol shared by the backend and the browser UI.
//!
//! Two channels on the same WebSocket:
//! - binary frames (`FrameHeader` + encoded payload) for display output
//! - JSON messages (`Event` backend→UI, `Command` UI→backend)

use serde::{Deserialize, Serialize};

/// Magic for binary frame headers: `"GHFR"`.
pub const FRAME_MAGIC: [u8; 4] = *b"GHFR";

/// Encoded payload formats. Selected per frame via `FrameHeader::codec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Codec {
    Jpeg = 0,
    Vp9 = 1,
    H264 = 2,
}

impl Codec {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Jpeg),
            1 => Some(Self::Vp9),
            2 => Some(Self::H264),
            _ => None,
        }
    }
}

/// Binary frame header. Little-endian. Followed immediately by the payload.
///
/// ```text
/// offset  size  field
/// 0       4     magic ("GHFR")
/// 4       1     codec (Codec as u8)
/// 5       3     padding (zero)
/// 8       4     width (u32 LE)
/// 12      4     height (u32 LE)
/// 16      8     seq (u64 LE)
/// 24      ...   payload
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub codec: Codec,
    pub width: u32,
    pub height: u32,
    pub seq: u64,
}

impl FrameHeader {
    pub const SIZE: usize = 24;

    pub fn encode(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&FRAME_MAGIC);
        buf[4] = self.codec as u8;
        buf[8..12].copy_from_slice(&self.width.to_le_bytes());
        buf[12..16].copy_from_slice(&self.height.to_le_bytes());
        buf[16..24].copy_from_slice(&self.seq.to_le_bytes());
        buf
    }
}

/// Backend → UI events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// Server is up and accepts commands.
    Ready { version: String },
    /// Generic log line the UI can surface.
    Log { level: String, message: String },
    /// Guest lifecycle transition.
    Guest { state: GuestState },
    /// Result of a `Command::Resize`.
    Resize { w: u32, h: u32, accepted: bool, reason: Option<String> },
    /// Error to surface to the user.
    Error { message: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuestState {
    Off,
    Booting,
    Running,
    Paused,
    Error,
}

/// UI → backend commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Boot a guest from a server-side image path.
    Boot { image: String },
    Pause,
    Resume,
    Snapshot { path: Option<String> },
    Resize { w: u32, h: u32, dpr: f32, mode: ResizeMode },
    Key { code: String, down: bool },
    Mouse { x: i32, y: i32, button: Option<u8>, down: Option<bool> },
    /// Round-trip liveness check.
    Ping { t: u64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResizeMode {
    Fit,
    Fill,
    Stretch,
}
