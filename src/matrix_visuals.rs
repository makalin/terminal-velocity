use bevy::prelude::*;
use crate::data::NUM_CHANNELS;

pub struct MatrixVisualsPlugin;

impl Plugin for MatrixVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_matrix_drops, update_drops));
    }
}

#[derive(Component)]
struct MatrixDrop {
    speed: f32,
    timer: Timer,
}

// Listen to AudioCommands to spawn visuals effectively "synced" with audio
// Note: We need a way to intercept the commands or listen to a similar event.
// Since AudioEngine consumes receiver, we might need a separate Event for visuals
// OR we can just check ProjectData state, but that's per-frame polling.
// BETTER: Let's create a visual event that Playback/Editor emits alongside AudioCommand.
// For now, let's piggyback on a new event `VisualNoteEvent`.

#[derive(Event)]
pub struct VisualNoteEvent {
    pub note_name: String,
    pub channel: usize,
}

fn spawn_matrix_drops(
    mut commands: Commands,
    mut events: EventReader<VisualNoteEvent>,
) {
    // Only show Matrix Rain in View/Play mode (Not Edit)
    // The user said: "while editing show all details", "matrix rain like play"
    // So if EditMode::Edit -> Show Grid (handled in UI), Hide Rain?
    // Or Rain can be background?
    // The request said "while playing... show only initial letter... falling like Matrix code"
    // So we assume this is active when playing.
    
    // We'll spawn regardless, visibility can be handled or we just let them fall behind the UI if needed.
    // But typically "Matrix like play" implies the whole screen is the visual.
    
    for event in events.read() {
        // Calculate X position based on Channel (0-7)
        // Screen width approx 1280. 8 Channels.
        // Spread them out.
        let lane_width = 1280.0 / NUM_CHANNELS as f32;
        let x = (event.channel as f32 * lane_width) - (1280.0 / 2.0) + (lane_width / 2.0);
        let y = 360.0; // Top of screen (approx)

        let initial = event.note_name.chars().next().unwrap_or('?').to_string();

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    initial, 
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.0, 1.0, 0.0), // MATRIX GREEN
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            MatrixDrop {
                speed: 300.0,
                timer: Timer::from_seconds(5.0, TimerMode::Once),
            },
        ));
    }
}

fn update_drops(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MatrixDrop, &mut Text)>,
) {
    for (entity, mut transform, mut drop, mut text) in query.iter_mut() {
        // Fall
        transform.translation.y -= drop.speed * time.delta_seconds();

        // Matrix Glitch (Randomly change char sometimes?) - Simplified for now
        // Fade out?
        
        let alpha = drop.timer.remaining_secs() / 5.0;
        for section in text.sections.iter_mut() {
            section.style.color.set_a(alpha);
        }

        drop.timer.tick(time.delta());
        if drop.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
