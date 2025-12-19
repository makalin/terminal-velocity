use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const NUM_CHANNELS: usize = 16;
pub const ROWS_PER_PATTERN: usize = 64;

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct ProjectData {
    pub bpm: u32,
    pub patterns: Vec<Pattern>,
    pub current_pattern: usize,
    pub playing: bool,
    pub current_row: usize,
    pub current_channel: usize,
    pub speed: u32,
    
    // Editor State
    pub current_octave: u8,
    pub current_instrument: u8,
    pub auto_advance: usize, 
    
    // Pro Features (Metadata)
    pub tracks: Vec<TrackConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackConfig {
    pub name: String,
    pub index: usize,
    pub muted: bool,
    pub solo: bool,
    pub volume: f32,
}

impl Default for TrackConfig {
    fn default() -> Self {
        Self {
            name: "Track".to_string(),
            index: 0,
            muted: false,
            solo: false,
            volume: 1.0,
        }
    }
}

impl Default for ProjectData {
    fn default() -> Self {
        // Initialize tracks with professional names
        let mut tracks = Vec::with_capacity(NUM_CHANNELS);
        let track_names = vec![
            "KICK", "SNARE", "BASSLINE", "LEAD SYNTH",
            "PAD", "ARP", "PLUCK", "FX",
            "TRACK 9", "TRACK 10", "TRACK 11", "TRACK 12",
            "TRACK 13", "TRACK 14", "TRACK 15", "TRACK 16"
        ];
        
        for i in 0..NUM_CHANNELS {
            let name = track_names.get(i)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("TRACK {}", i + 1));
            tracks.push(TrackConfig {
                name,
                index: i,
                ..default()
            });
        }

        Self {
            bpm: 140,
            patterns: vec![Pattern::default()], 
            current_pattern: 0,
            playing: false,
            current_row: 0,
            current_channel: 0,
            speed: 6,
            current_octave: 4,
            current_instrument: 1,
            auto_advance: 1,
            tracks,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pattern {
    pub rows: Vec<Row>,
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            rows: vec![Row::default(); ROWS_PER_PATTERN],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct Row {
    pub channels: [ChannelData; NUM_CHANNELS],
}

impl Default for Row {
    fn default() -> Self {
        Self {
            channels: [ChannelData::default(); NUM_CHANNELS],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct ChannelData {
    pub note: Option<u8>, // MIDI Note Number (0-127)
    pub instrument: Option<u8>,
    pub volume: Option<u8>, // 0-64
    pub effect: Option<EffectType>, 
    pub effect_value: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum EffectType {
    None,
    PitchSlide, // 1xx
    Arpeggio,   // 0xy
    VolumeSlide,// Axy
    // Add more as needed
}

// Marker component for valid "Visual" note if we need to spawn entities (optional in data-driven UI)
// For a tracker, we usually render the grid directly from data, so we might not need many entities.
// Keeping this just in case.
#[derive(Component)]
#[allow(dead_code)]
pub struct VisualCursor {
    pub x: usize,
    pub y: usize,
}

#[derive(Event, Default)]
pub struct ProjectLoadedEvent;
