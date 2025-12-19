use bevy::prelude::*;
use crate::data::{ProjectData, ROWS_PER_PATTERN, NUM_CHANNELS};
use crate::audio_engine::{AudioCommand, AudioEngine, MidiInputEvent};
use crate::matrix_visuals::VisualNoteEvent;

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum EditMode {
    #[default]
    View,
    Edit,
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectData>()
           .init_resource::<EditMode>()
           .add_systems(Update, (handle_keyboard, handle_midi_input));
    }
}

// ...

fn handle_keyboard(
    mut project: ResMut<ProjectData>,
    mut edit_mode: ResMut<EditMode>,
    keyboard: Res<ButtonInput<KeyCode>>,
    audio: Res<AudioEngine>,
    mut visual_events: EventWriter<VisualNoteEvent>,
) {
    // ... (Navigation, Play/Stop, Mode Toggle unchanged)

    // 1. Navigation
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if project.current_row > 0 { project.current_row -= 1; }
        else { project.current_row = ROWS_PER_PATTERN - 1; }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        project.current_row = (project.current_row + 1) % ROWS_PER_PATTERN;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        if project.current_channel > 0 { project.current_channel -= 1; }
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        if project.current_channel < NUM_CHANNELS - 1 { project.current_channel += 1; }
    }

    // 2. Play/Stop
    if keyboard.just_pressed(KeyCode::Space) {
        project.playing = !project.playing;
    }

    // 3. Edit Mode Toggle
    if keyboard.just_pressed(KeyCode::Enter) {
        *edit_mode = match *edit_mode {
            EditMode::View => EditMode::Edit,
            EditMode::Edit => EditMode::View,
        };
    }

    // 4. Note Entry (Only in Edit Mode)
    if *edit_mode == EditMode::Edit {
        if let Some(note_offset) = key_to_note(keyboard.get_just_pressed().next()) {
            let octave = project.current_octave;
            let note = (octave as u8 * 12) + note_offset;
            
            let current_inst = project.current_instrument;
            let auto_advance = project.auto_advance;

            let pattern_idx = project.current_pattern;
            let row_idx = project.current_row;
            let ch_idx = project.current_channel;
            
            if pattern_idx < project.patterns.len() {
                // Get track instrument before borrowing cell
                let inst = if let Some(track) = project.tracks.get(ch_idx) {
                    track.index as u8
                } else {
                    current_inst
                };
                
                let cell = &mut project.patterns[pattern_idx].rows[row_idx].channels[ch_idx];
                cell.note = Some(note);
                cell.instrument = Some(inst);
                cell.volume = Some(64);

                audio.sender.send(AudioCommand::PlayNote {
                    note,
                    instrument: inst,
                    velocity: 127,
                    channel: ch_idx, 
                }).ok();

                visual_events.send(VisualNoteEvent {
                    note_name: note_name(note),
                    channel: ch_idx,
                });

                project.current_row = (row_idx + auto_advance) % ROWS_PER_PATTERN;
            }
        }
        
        // Delete Note logic unchanged...
        if keyboard.just_pressed(KeyCode::Backspace) || keyboard.just_pressed(KeyCode::Delete) {
             let pattern_idx = project.current_pattern;
             let row_idx = project.current_row;
             let ch_idx = project.current_channel;
             if pattern_idx < project.patterns.len() {
                 let cell = &mut project.patterns[pattern_idx].rows[row_idx].channels[ch_idx];
                 cell.note = None;
                 cell.instrument = None;
                 cell.volume = None;
             }
        }
    }
}

fn handle_midi_input(
    mut events: EventReader<MidiInputEvent>,
    mut project: ResMut<ProjectData>,
    edit_mode: Res<EditMode>,
    audio: Res<AudioEngine>,
    mut visual_events: EventWriter<VisualNoteEvent>,
) {
    if *edit_mode != EditMode::Edit { return; }

    for event in events.read() {
        if event.status == 0x90 && event.velocity > 0 { // Note On
            let note = event.note;
            
            let current_inst = project.current_instrument;
            let auto_advance = project.auto_advance;

            let pattern_idx = project.current_pattern;
            let row_idx = project.current_row;
            let ch_idx = project.current_channel;
            
             if pattern_idx < project.patterns.len() {
                // Get track instrument before borrowing cell
                let inst = if let Some(track) = project.tracks.get(ch_idx) {
                    track.index as u8
                } else {
                    current_inst
                };
                
                let cell = &mut project.patterns[pattern_idx].rows[row_idx].channels[ch_idx];
                cell.note = Some(note);
                cell.instrument = Some(inst);
                cell.volume = Some(event.velocity / 2);

                audio.sender.send(AudioCommand::PlayNote {
                    note,
                    instrument: inst,
                    velocity: event.velocity,
                    channel: ch_idx, 
                }).ok();

                visual_events.send(VisualNoteEvent {
                    note_name: note_name(note),
                    channel: ch_idx,
                });

                project.current_row = (row_idx + auto_advance) % ROWS_PER_PATTERN;
            }
        }
    }
}

fn note_name(midi_note: u8) -> String {
    let notes = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"];
    let note_idx = (midi_note % 12) as usize;
    notes[note_idx].to_string()
}

fn key_to_note(key: Option<&KeyCode>) -> Option<u8> {
    match key {
        Some(KeyCode::KeyZ) => Some(0),
        Some(KeyCode::KeyS) => Some(1),
        Some(KeyCode::KeyX) => Some(2),
        Some(KeyCode::KeyD) => Some(3),
        Some(KeyCode::KeyC) => Some(4),
        Some(KeyCode::KeyV) => Some(5),
        Some(KeyCode::KeyG) => Some(6),
        Some(KeyCode::KeyB) => Some(7),
        Some(KeyCode::KeyH) => Some(8),
        Some(KeyCode::KeyN) => Some(9),
        Some(KeyCode::KeyJ) => Some(10),
        Some(KeyCode::KeyM) => Some(11),
        Some(KeyCode::Comma) => Some(12),
        _ => None,
    }
}
