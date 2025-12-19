use bevy::prelude::*;
use crate::data::{ProjectData, ROWS_PER_PATTERN};
use crate::audio_engine::{AudioCommand, AudioEngine};
use crate::matrix_visuals::VisualNoteEvent;

pub struct PlaybackPlugin;

impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, playback_system);
    }
}

fn playback_system(
    mut project: ResMut<ProjectData>,
    time: Res<Time>,
    audio: Res<AudioEngine>, 
    mut visual_events: EventWriter<VisualNoteEvent>,
    mut row_timer: Local<f32>,
    mut last_bpm: Local<u32>,
) {
    // Send BPM changes to audio engine
    if *last_bpm != project.bpm {
        let _ = audio.sender.send(crate::audio_engine::AudioCommand::SetBpm(project.bpm as f32));
        *last_bpm = project.bpm;
    }
    if !project.playing { 
        *row_timer = 0.0;
        return; 
    }

    let ticks_per_row = project.speed as f32;
    // Standard Tracker Formula: Time = 2.5 / BPM per tick
    let row_duration = ticks_per_row * (2.5 / project.bpm as f32);

    *row_timer += time.delta_seconds();

    while *row_timer >= row_duration {
        *row_timer -= row_duration;

        // 1. Play Current Row
        let pattern_idx = project.current_pattern;
        if let Some(pattern) = project.patterns.get(pattern_idx) {
             if let Some(row) = pattern.rows.get(project.current_row) {
                 for (ch_idx, cell) in row.channels.iter().enumerate() {
                     // Check for Note
                     if let Some(note) = cell.note {
                         // Check if track is muted
                         if let Some(track) = project.tracks.get(ch_idx) {
                             if track.muted {
                                 continue;
                             }
                         }
                         
                         // Default params - use track instrument if available
                         let inst = if let Some(track) = project.tracks.get(ch_idx) {
                             track.index as u8
                         } else {
                             cell.instrument.unwrap_or(project.current_instrument)
                         };
                         let vol = cell.volume.unwrap_or(64);
                         
                         // Apply track volume
                         let track_vol = if let Some(track) = project.tracks.get(ch_idx) {
                             track.volume
                         } else {
                             1.0
                         };
                         let final_vol = ((vol as f32 / 127.0) * track_vol * 127.0) as u8;
                         
                         // Audio Command
                         let _ = audio.sender.send(AudioCommand::PlayNote {
                             note,
                             instrument: inst,
                             velocity: final_vol.min(127), 
                             channel: ch_idx,
                         });

                         // Visual Event
                         visual_events.send(VisualNoteEvent {
                             note_name: note_name(note),
                             channel: ch_idx,
                         });
                     }
                 }
             }
        }
        
        // 2. Advance Row
        project.current_row += 1;
        if project.current_row >= ROWS_PER_PATTERN {
            project.current_row = 0;
            // Loop pattern for now
        }
    }
}

fn note_name(midi_note: u8) -> String {
    let notes = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"];
    let note_idx = (midi_note % 12) as usize;
    notes[note_idx].to_string()
}
