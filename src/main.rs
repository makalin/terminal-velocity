use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod audio_engine;
mod ui_overlay;
mod data;
mod editor;
mod persistence;
mod playback;

mod matrix_visuals;
pub mod ui_widgets;
mod themes;

use audio_engine::AudioPlugin;
use ui_overlay::UiOverlayPlugin;
use editor::EditorPlugin;
use playback::PlaybackPlugin;
use matrix_visuals::{MatrixVisualsPlugin, VisualNoteEvent};

fn main() {
    App::new()
        .add_event::<data::ProjectLoadedEvent>()
        .add_event::<VisualNoteEvent>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Terminal Velocity (Matrix Edition)".into(),
                resolution: (1280., 720.).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(UiOverlayPlugin)
        .add_plugins(EditorPlugin)
        .add_plugins(PlaybackPlugin)
        .add_plugins(MatrixVisualsPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
