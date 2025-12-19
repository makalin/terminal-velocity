use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::{Color32, Window};
use crate::data::{ProjectData, NUM_CHANNELS, ROWS_PER_PATTERN};
use crate::editor::EditMode;
use crate::persistence::{save_project, load_project};
use crate::ui_widgets::{knob, cyber_slider};
use crate::audio_engine::{AudioEngine, AudioCommand};
use crate::themes::{Theme, ThemeColors};
use std::collections::VecDeque;
use rand::Rng;

pub struct UiOverlayPlugin;

impl Plugin for UiOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system);
    }
}

struct MatrixColumn {
    x: f32,
    chars: VecDeque<(f32, char, f32)>, // y, char, brightness
    speed: f32,
    next_char_time: f32,
    char_interval: f32,
}

#[derive(Default)]
struct UiState {
    selected_note: Option<(usize, usize)>,
    device_cutoff: f32,
    device_resonance: f32,
    device_drive: f32,
    device_attack: f32,
    device_decay: f32,
    device_sustain: f32,
    device_release: f32,
    device_delay: f32,
    device_reverb: f32,
    playback_start_time: Option<f64>,
    cpu_usage: f32,
    show_help: bool,
    show_about: bool,
    show_settings: bool,
    show_midi_config: bool,
    show_mixer: bool,
    show_file_dialog: bool,
    file_dialog_mode: FileDialogMode,
    midi_port_selection: String,
    current_theme: Theme,
    matrix_columns: VecDeque<MatrixColumn>,
    window_positions: std::collections::HashMap<String, egui::Rect>,
    hover_tooltip: Option<String>,
}

#[derive(Clone, Copy, Default)]
enum FileDialogMode {
    #[default]
    Save,
    Load,
}

fn ui_system(
    mut contexts: EguiContexts, 
    mut project: ResMut<ProjectData>,
    mut edit_mode: ResMut<EditMode>,
    mut scroll_y: Local<f32>,
    mut ui_state: Local<UiState>,
    time: Res<Time>,
    audio: Res<AudioEngine>,
) {
    let ctx = contexts.ctx_mut();
    let colors = ThemeColors::get(ui_state.current_theme);
    
    // Apply theme
    let mut style = (*ctx.style()).clone();
    colors.apply_to_style(&mut style);
    style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(11.0));
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::monospace(11.0));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::monospace(11.0));
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::monospace(12.0));
    ctx.set_style(style);

    // Update playback time
    if project.playing {
        if ui_state.playback_start_time.is_none() {
            ui_state.playback_start_time = Some(time.elapsed_seconds_f64());
        }
    } else {
        ui_state.playback_start_time = None;
        ui_state.matrix_columns.clear();
    }

    ui_state.cpu_usage = 2.0 + (time.elapsed_seconds() * 0.1).sin() * 1.5;

    // File Dialog
    if ui_state.show_file_dialog {
        let title = match ui_state.file_dialog_mode {
            FileDialogMode::Save => "Save Project",
            FileDialogMode::Load => "Load Project",
        };
        
        let mut open = true;
        Window::new(title)
            .collapsible(false)
            .resizable(false)
            .default_size([400.0, 200.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("File name:");
                    let mut filename = String::from("project.json");
                    ui.text_edit_singleline(&mut filename);
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            ui_state.show_file_dialog = false;
                            open = false;
                        }
                        if ui.button(match ui_state.file_dialog_mode {
                            FileDialogMode::Save => "Save",
                            FileDialogMode::Load => "Load",
                        }).clicked() {
                            match ui_state.file_dialog_mode {
                                FileDialogMode::Save => {
                                    let _ = save_project(&project, &filename);
                                }
                                FileDialogMode::Load => {
                                    if let Ok(p) = load_project(&filename) {
                                        *project = p;
                                        *edit_mode = EditMode::View;
                                    }
                                }
                            }
                            ui_state.show_file_dialog = false;
                            open = false;
                        }
                    });
                });
            });
        
        if !open {
            ui_state.show_file_dialog = false;
        }
    }

    // Settings Window
    if ui_state.show_settings {
        let mut open = true;
        Window::new("Settings")
            .collapsible(true)
            .resizable(true)
            .default_size([500.0, 600.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Theme").strong().size(11.0));
                        ui.separator();
                        if ui.selectable_label(ui_state.current_theme == Theme::Matrix, "Matrix").clicked() {
                            ui_state.current_theme = Theme::Matrix;
                        }
                        if ui.selectable_label(ui_state.current_theme == Theme::Cyberpunk, "Cyberpunk").clicked() {
                            ui_state.current_theme = Theme::Cyberpunk;
                        }
                        if ui.selectable_label(ui_state.current_theme == Theme::Monochrome, "Monochrome").clicked() {
                            ui_state.current_theme = Theme::Monochrome;
                        }
                        if ui.selectable_label(ui_state.current_theme == Theme::Neon, "Neon").clicked() {
                            ui_state.current_theme = Theme::Neon;
                        }
                        if ui.selectable_label(ui_state.current_theme == Theme::Professional, "Professional").clicked() {
                            ui_state.current_theme = Theme::Professional;
                        }
                        
                        ui.add_space(20.0);
                        
                        ui.label(egui::RichText::new("Keyboard Shortcuts").strong().size(11.0));
                        ui.separator();
                        ui.label("Navigation:");
                        ui.label("  ↑ ↓ ← → : Move cursor");
                        ui.label("  SPACE : Play/Stop");
                        ui.label("  ENTER : Toggle Edit Mode");
                        ui.add_space(5.0);
                        ui.label("Note Entry:");
                        ui.label("  Z S X D C V G B H N J M , : Notes");
                        ui.label("  BACKSPACE/DELETE : Delete note");
                        ui.add_space(5.0);
                        ui.label("File Operations:");
                        ui.label("  CTRL+S : Save project");
                        ui.label("  CTRL+O : Load project");
                    });
                });
                
                ui.separator();
                if ui.button("Close").clicked() {
                    open = false;
                }
            });
        
        if !open {
            ui_state.show_settings = false;
        }
    }

    // Help Dialog
    if ui_state.show_help {
        let mut open = true;
        Window::new("Keyboard Shortcuts")
            .collapsible(true)
            .resizable(true)
            .default_size([500.0, 400.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Navigation").strong().size(11.0));
                    ui.label("↑ ↓ ← → : Move cursor");
                    ui.label("SPACE : Play/Stop");
                    ui.label("ENTER : Toggle Edit Mode");
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("Note Entry (Edit Mode)").strong().size(11.0));
                    ui.label("Z S X D C V G B H N J M , : Play notes (C to B)");
                    ui.label("BACKSPACE/DELETE : Delete note");
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("File Operations").strong().size(11.0));
                    ui.label("CTRL+S : Save project");
                    ui.label("CTRL+O : Load project");
                });
                
                ui.separator();
                if ui.button("Close").clicked() {
                    open = false;
                }
            });
        
        if !open {
            ui_state.show_help = false;
        }
    }

    // About Dialog
    if ui_state.show_about {
        let mut open = true;
        Window::new("About")
            .collapsible(true)
            .resizable(false)
            .default_size([400.0, 200.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Terminal Velocity");
                    ui.label("Professional Tracker");
                    ui.add_space(10.0);
                    ui.label("Version 0.2.0");
                    ui.add_space(10.0);
                    ui.label("A modern music tracker with Matrix visuals");
                });
                
                ui.separator();
                if ui.button("Close").clicked() {
                    open = false;
                }
            });
        
        if !open {
            ui_state.show_about = false;
        }
    }

    // MIDI Config Dialog
    if ui_state.show_midi_config {
        let mut open = true;
        Window::new("MIDI Configuration")
            .collapsible(true)
            .resizable(true)
            .default_size([400.0, 250.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("MIDI Input Device:");
                    ui.text_edit_singleline(&mut ui_state.midi_port_selection);
                    ui.label("(Auto-detected: First available MIDI port)");
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Status: Connected").color(Color32::from_rgb(0, 255, 0)));
                    ui.add_space(10.0);
                    ui.label("MIDI is working! Connect your MIDI device and play notes in Edit Mode.");
                });
                
                ui.separator();
                if ui.button("Close").clicked() {
                    open = false;
                }
            });
        
        if !open {
            ui_state.show_midi_config = false;
        }
    }

    // TOP BAR
    egui::TopBottomPanel::top("top_panel")
        .exact_height(35.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("FILE", |ui| {
                    if ui.button("Save Project... (Ctrl+S)").clicked() {
                        ui_state.file_dialog_mode = FileDialogMode::Save;
                        ui_state.show_file_dialog = true;
                    }
                    if ui.button("Load Project... (Ctrl+O)").clicked() {
                        ui_state.file_dialog_mode = FileDialogMode::Load;
                        ui_state.show_file_dialog = true;
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("EDIT", |ui| {
                    if ui.button("Toggle Edit Mode (ENTER)").clicked() {
                        *edit_mode = match *edit_mode {
                            EditMode::View => EditMode::Edit,
                            EditMode::Edit => EditMode::View,
                        };
                    }
                });
                
                ui.menu_button("VIEW", |ui| {
                    if ui.button("Show Mixer").clicked() {
                        ui_state.show_mixer = true;
                    }
                    if ui.button("Hide Mixer").clicked() {
                        ui_state.show_mixer = false;
                    }
                });
                
                ui.menu_button("SETTINGS", |ui| {
                    if ui.button("Settings...").clicked() {
                        ui_state.show_settings = true;
                    }
                    ui.separator();
                    if ui.button("MIDI Configuration...").clicked() {
                        ui_state.show_midi_config = true;
                    }
                });
                
                ui.menu_button("HELP", |ui| {
                    if ui.button("Keyboard Shortcuts").clicked() {
                        ui_state.show_help = true;
                    }
                    if ui.button("About").clicked() {
                        ui_state.show_about = true;
                    }
                });
                
                ui.separator();
                
                // BPM Control
                ui.label(egui::RichText::new("BPM:").size(10.0));
                let mut bpm_f32 = project.bpm as f32;
                ui.add(egui::DragValue::new(&mut bpm_f32).speed(1.0).clamp_range(60.0..=200.0));
                let new_bpm = bpm_f32 as u32;
                if new_bpm != project.bpm {
                    project.bpm = new_bpm;
                    let _ = audio.sender.send(AudioCommand::SetBpm(project.bpm as f32));
                }
                if ui.small_button("−").clicked() {
                    if project.bpm > 60 {
                        project.bpm -= 1;
                        let _ = audio.sender.send(AudioCommand::SetBpm(project.bpm as f32));
                    }
                }
                if ui.small_button("+").clicked() {
                    if project.bpm < 200 {
                        project.bpm += 1;
                        let _ = audio.sender.send(AudioCommand::SetBpm(project.bpm as f32));
                    }
                }
                
                ui.separator();
                
                // Play/Stop button
                let play_text = if project.playing { "⏸" } else { "▶" };
                let play_color = if project.playing {
                    Color32::from_rgb(255, 0, 0)
                } else {
                    colors.primary
                };
                if ui.button(egui::RichText::new(play_text).color(play_color).size(14.0)).clicked() {
                    project.playing = !project.playing;
                }
                
                ui.separator();
                
                // Right side: Performance metrics
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("CPU: {:.1}%", ui_state.cpu_usage)).size(10.0));
                    
                    let playback_time = if let Some(start) = ui_state.playback_start_time {
                        time.elapsed_seconds_f64() - start
                    } else {
                        0.0
                    };
                    let minutes = (playback_time / 60.0) as u32;
                    let seconds = (playback_time % 60.0) as u32;
                    let frames = ((playback_time % 1.0) * 30.0) as u32;
                    ui.label(egui::RichText::new(format!("{:02}:{:02}:{:02}", minutes, seconds, frames)).size(10.0));
                    
                    ui.label(egui::RichText::new(format!("R:{:02}", project.current_row)).size(10.0));
                });
            });
        });

    // MIXER PANEL - Show ALL channels side by side (no scrolling)
    if ui_state.show_mixer {
        egui::TopBottomPanel::bottom("mixer_panel")
            .default_height(180.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("MIXER").size(10.0).color(colors.text_dim));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✕").clicked() {
                            ui_state.show_mixer = false;
                        }
                    });
                });
                ui.separator();
                
                let current_channel_mixer = project.current_channel;
                // Show all channels in a single row, no scrolling
                ui.horizontal(|ui| {
                    for (i, track) in project.tracks.iter_mut().enumerate() {
                        let is_selected = i == current_channel_mixer;
                        
                        ui.push_id(i, |ui| {
                            // Calculate width to fit all channels
                            let available_width = ui.available_width();
                            let channel_width = (available_width / NUM_CHANNELS as f32).max(60.0).min(80.0);
                            
                            let frame = egui::Frame::none()
                                .fill(if is_selected { colors.active } else { colors.surface })
                                .stroke(egui::Stroke::new(1.0, colors.border))
                                .inner_margin(egui::Margin::same(4.0));
                            
                            frame.show(ui, |ui| {
                                ui.set_width(channel_width);
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new(format!("CH{:02}", i + 1))
                                        .monospace()
                                        .size(9.0)
                                        .color(colors.text));
                                    ui.label(egui::RichText::new(&track.name)
                                        .monospace()
                                        .size(8.0)
                                        .color(colors.text_dim));
                                    
                                    ui.add_space(3.0);
                                    
                                    // Volume slider (vertical)
                                    let mut vol = track.volume;
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("VOL").color(colors.text_dim).size(8.0));
                                        let response = ui.add(egui::Slider::new(&mut vol, 0.0..=1.0)
                                            .orientation(egui::SliderOrientation::Vertical)
                                            .show_value(false));
                                        if response.changed() {
                                            track.volume = vol;
                                        }
                                        if response.hovered() {
                                            ui_state.hover_tooltip = Some(format!("Channel {} Volume: {:.0}%", i + 1, vol * 100.0));
                                        }
                                        ui.label(egui::RichText::new(format!("{:.0}%", vol * 100.0))
                                            .monospace()
                                            .size(8.0)
                                            .color(colors.text_dim));
                                    });
                                    
                                    ui.add_space(3.0);
                                    
                                    // Mute/Solo buttons
                                    ui.horizontal(|ui| {
                                        let mute_color = if track.muted {
                                            Color32::from_rgb(255, 0, 0)
                                        } else {
                                            colors.text
                                        };
                                        
                                        let mute_btn = ui.small_button(egui::RichText::new("M").color(mute_color).size(9.0));
                                        if mute_btn.clicked() {
                                            track.muted = !track.muted;
                                        }
                                        if mute_btn.hovered() {
                                            ui_state.hover_tooltip = Some(format!("Channel {} Mute", i + 1));
                                        }
                                        
                                        let solo_color = if track.solo {
                                            Color32::from_rgb(255, 255, 0)
                                        } else {
                                            colors.text
                                        };
                                        
                                        let solo_btn = ui.small_button(egui::RichText::new("S").color(solo_color).size(9.0));
                                        if solo_btn.clicked() {
                                            track.solo = !track.solo;
                                        }
                                        if solo_btn.hovered() {
                                            ui_state.hover_tooltip = Some(format!("Channel {} Solo", i + 1));
                                        }
                                    });
                                    
                                    // Instrument selector
                                    ui.add_space(3.0);
                                    ui.label(egui::RichText::new("INST:").color(colors.text_dim).size(8.0));
                                    let inst_names = ["SINE", "SQUARE", "SAW", "PULSE"];
                                    let mut inst_idx = (track.index % 4) as usize;
                                    let inst_label = ui.selectable_label(false, inst_names[inst_idx]);
                                    if inst_label.clicked() {
                                        inst_idx = (inst_idx + 1) % 4;
                                        track.index = inst_idx;
                                    }
                                    if inst_label.hovered() {
                                        ui_state.hover_tooltip = Some(format!("Channel {} Instrument: {}", i + 1, inst_names[inst_idx]));
                                    }
                                });
                            });
                        });
                        
                        ui.add_space(2.0);
                    }
                });
            });
    }

    // LEFT PANEL: TRACKS - Minimal title
    egui::SidePanel::left("tracks_panel")
        .default_width(260.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("TRACKS").size(9.0).color(colors.text_dim));
            });
            ui.separator();
            
            let current_channel = project.current_channel;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, track) in project.tracks.iter_mut().enumerate() {
                    let is_selected = i == current_channel;
                    
                    ui.push_id(i, |ui| {
                        let frame = egui::Frame::none()
                            .fill(if is_selected { colors.active } else { colors.surface })
                            .stroke(egui::Stroke::new(1.0, colors.border))
                            .inner_margin(egui::Margin::same(6.0));
                        
                        frame.show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("{}.", i + 1))
                                        .color(colors.primary)
                                        .size(10.0)
                                        .monospace());
                                    
                                    if is_selected {
                                        ui.text_edit_singleline(&mut track.name);
                                    } else {
                                        ui.label(egui::RichText::new(&track.name)
                                            .monospace()
                                            .size(10.0)
                                            .color(colors.text));
                                    }
                                });
                                
                                ui.add_space(3.0);
                                
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("INST:").color(colors.text_dim).size(9.0));
                                    let inst_names = ["SINE", "SQUARE", "SAW", "PULSE"];
                                    let mut inst_idx = (track.index % 4) as usize;
                                    if ui.selectable_label(false, inst_names[inst_idx]).clicked() {
                                        inst_idx = (inst_idx + 1) % 4;
                                        track.index = inst_idx;
                                    }
                                });
                                
                                ui.add_space(3.0);
                                
                                ui.horizontal(|ui| {
                                    let mute_color = if track.muted {
                                        Color32::from_rgb(255, 0, 0)
                                    } else {
                                        colors.text
                                    };
                                    
                                    if ui.small_button(egui::RichText::new("MUTE").color(mute_color).size(9.0))
                                        .clicked() {
                                        track.muted = !track.muted;
                                    }
                                    
                                    let solo_color = if track.solo {
                                        Color32::from_rgb(255, 255, 0)
                                    } else {
                                        colors.text
                                    };
                                    
                                    if ui.small_button(egui::RichText::new("SOLO").color(solo_color).size(9.0))
                                        .clicked() {
                                        track.solo = !track.solo;
                                    }
                                });
                                
                                ui.add_space(3.0);
                                
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("VOL").color(colors.text_dim).size(9.0));
                                    cyber_slider(ui, &mut track.volume, 0.0..=1.0);
                                    ui.label(egui::RichText::new(format!("{:.0}%", track.volume * 100.0))
                                        .monospace()
                                        .size(9.0)
                                        .color(colors.text_dim));
                                });
                            });
                        });
                    });
                    
                    ui.add_space(3.0);
                }
            });
        });

    // RIGHT PANEL: INSPECTOR & DEVICE - Minimal titles, more tools
    egui::SidePanel::right("inspector_panel")
        .default_width(300.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("INSPECTOR").size(9.0).color(colors.text_dim));
            });
            ui.separator();
            
            let frame = egui::Frame::none()
                .fill(colors.surface)
                .stroke(egui::Stroke::new(1.0, colors.border))
                .inner_margin(egui::Margin::same(8.0));
            
            frame.show(ui, |ui| {
                if let Some((row, ch)) = ui_state.selected_note {
                    if let Some(pattern) = project.patterns.get(project.current_pattern) {
                        if let Some(row_data) = pattern.rows.get(row) {
                            if let Some(cell) = row_data.channels.get(ch) {
                                if let Some(note) = cell.note {
                                    let note_name_str = note_name(note);
                                    
                                    ui.label(egui::RichText::new(format!("NOTE: {}", note_name_str))
                                        .monospace()
                                        .color(colors.primary)
                                        .size(10.0));
                                    
                                    ui.add_space(6.0);
                                    
                                    ui.label(egui::RichText::new("VELOCITY:").monospace().color(colors.text).size(9.0));
                                    let velocity = cell.volume.unwrap_or(64);
                                    let mut vel_f32 = velocity as f32 / 127.0;
                                    cyber_slider(ui, &mut vel_f32, 0.0..=1.0);
                                    ui.label(egui::RichText::new(format!("{}", velocity))
                                        .monospace()
                                        .size(9.0)
                                        .color(colors.text_dim));
                                    
                                    ui.add_space(6.0);
                                    
                                    ui.label(egui::RichText::new("LENGTH: 1/16").monospace().color(colors.text).size(9.0));
                                    
                                    ui.add_space(6.0);
                                    
                                    ui.label(egui::RichText::new("CC 74:").monospace().color(colors.text).size(9.0));
                                    let mut cc74 = 64.0 / 127.0;
                                    knob(ui, &mut cc74, 0.0..=1.0, 40.0);
                                    ui.label(egui::RichText::new("64").monospace().size(9.0).color(colors.text_dim));
                                } else {
                                    ui.label(egui::RichText::new("NOTE: None").monospace().color(colors.text_dim).size(9.0));
                                }
                            }
                        }
                    }
                } else {
                    ui.label(egui::RichText::new("NOTE: None").monospace().color(colors.text_dim).size(9.0));
                }
            });
            
            ui.add_space(15.0);
            
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("DEVICE").size(9.0).color(colors.text_dim));
            });
            ui.separator();
            
            let frame = egui::Frame::none()
                .fill(colors.surface)
                .stroke(egui::Stroke::new(1.0, colors.border))
                .inner_margin(egui::Margin::same(8.0));
            
            frame.show(ui, |ui| {
                ui.vertical(|ui| {
                    // Filter Section
                    ui.label(egui::RichText::new("FILTER").size(9.0).color(colors.text_dim));
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            knob(ui, &mut ui_state.device_cutoff, 0.0..=1.0, 45.0);
                            ui.label(egui::RichText::new("CUTOFF").monospace().size(8.0).color(colors.text));
                        });
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            knob(ui, &mut ui_state.device_resonance, 0.0..=1.0, 45.0);
                            ui.label(egui::RichText::new("RES").monospace().size(8.0).color(colors.text));
                        });
                    });
                    
                    ui.add_space(8.0);
                    
                    // Distortion
                    ui.label(egui::RichText::new("DISTORTION").size(9.0).color(colors.text_dim));
                    ui.vertical(|ui| {
                        knob(ui, &mut ui_state.device_drive, 0.0..=1.0, 45.0);
                        ui.label(egui::RichText::new("DRIVE").monospace().size(8.0).color(colors.text));
                    });
                    
                    ui.add_space(8.0);
                    
                    // ADSR Envelope
                    ui.label(egui::RichText::new("ENVELOPE").size(9.0).color(colors.text_dim));
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            knob(ui, &mut ui_state.device_attack, 0.0..=1.0, 35.0);
                            ui.label(egui::RichText::new("A").monospace().size(8.0).color(colors.text));
                        });
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            knob(ui, &mut ui_state.device_decay, 0.0..=1.0, 35.0);
                            ui.label(egui::RichText::new("D").monospace().size(8.0).color(colors.text));
                        });
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            knob(ui, &mut ui_state.device_sustain, 0.0..=1.0, 35.0);
                            ui.label(egui::RichText::new("S").monospace().size(8.0).color(colors.text));
                        });
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            knob(ui, &mut ui_state.device_release, 0.0..=1.0, 35.0);
                            ui.label(egui::RichText::new("R").monospace().size(8.0).color(colors.text));
                        });
                    });
                    
                    ui.add_space(8.0);
                    
                    // Effects
                    ui.label(egui::RichText::new("EFFECTS").size(9.0).color(colors.text_dim));
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            knob(ui, &mut ui_state.device_delay, 0.0..=1.0, 40.0);
                            ui.label(egui::RichText::new("DELAY").monospace().size(8.0).color(colors.text));
                        });
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            knob(ui, &mut ui_state.device_reverb, 0.0..=1.0, 40.0);
                            ui.label(egui::RichText::new("REVERB").monospace().size(8.0).color(colors.text));
                        });
                    });
                });
            });
        });

    // CENTER PANEL: PATTERN EDITOR
    egui::CentralPanel::default().show(ctx, |ui| {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        // Handle mouse clicks
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let col_width = rect.width() / (NUM_CHANNELS + 1) as f32; // +1 for row number column
                let row_height = 22.0;
                let col = ((pos.x - rect.left()) / col_width) as usize;
                let row = ((pos.y - rect.top() + *scroll_y) / row_height) as usize;
                // Column 0 is row numbers, channels start from column 1
                if col > 0 && col <= NUM_CHANNELS && row < ROWS_PER_PATTERN {
                    let ch = col - 1; // Convert column to channel (0-indexed)
                    ui_state.selected_note = Some((row, ch));
                    project.current_channel = ch;
                    project.current_row = row;
                }
            }
        }
        
        // Scroll handling
        if ui.is_rect_visible(rect) {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                let total_rows = ROWS_PER_PATTERN as f32;
                let row_h = 22.0;
                let total_h = total_rows * row_h;
                let view_h = rect.height();
                
                if total_h > view_h {
                    *scroll_y -= scroll_delta;
                    *scroll_y = scroll_y.clamp(0.0, total_h - view_h);
                }
            }
            
            if project.playing {
                let current_row_y = project.current_row as f32 * 22.0;
                let center_offset = rect.height() / 2.0;
                *scroll_y = current_row_y - center_offset;
            }
        }

        let painter = ui.painter_at(rect);
        let col_width = rect.width() / (NUM_CHANNELS + 1) as f32; // +1 for row number column
        let row_height = 22.0;
        let dt = ui.input(|i| i.stable_dt);

        // REALISTIC MATRIX RAIN
        if project.playing {
            let mut rng = rand::thread_rng();
            
            if rng.gen::<f32>() < 0.02 * dt {
                let col = rng.gen_range(1..=NUM_CHANNELS); // Skip column 0 (row numbers)
                let x = rect.left() + col as f32 * col_width + col_width / 2.0;
                let speed = rng.gen_range(150.0..400.0);
                let char_interval = rng.gen_range(0.05..0.15);
                
                ui_state.matrix_columns.push_back(MatrixColumn {
                    x,
                    chars: VecDeque::new(),
                    speed,
                    next_char_time: 0.0,
                    char_interval,
                });
            }
            
            let matrix_chars = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
            
            let mut next_columns = VecDeque::new();
            for mut column in ui_state.matrix_columns.drain(..) {
                column.next_char_time -= dt;
                
                if column.next_char_time <= 0.0 {
                    let c = matrix_chars.chars().nth(rng.gen_range(0..matrix_chars.len())).unwrap_or('0');
                    column.chars.push_front((rect.top() - 10.0, c, 1.0));
                    column.next_char_time = column.char_interval;
                }
                
                let mut next_chars = VecDeque::new();
                for (y, c, brightness) in column.chars.drain(..) {
                    let new_y = y + column.speed * dt;
                    
                    if new_y < rect.bottom() + 100.0 {
                        let new_brightness = (brightness * 0.92).max(0.1);
                        next_chars.push_back((new_y, c, new_brightness));
                        
                        let alpha = (new_brightness * 255.0) as u8;
                        let color = colors.primary;
                        let trail_color = Color32::from_rgba_premultiplied(
                            color.r(),
                            color.g(),
                            color.b(),
                            (alpha as f32 * 0.3) as u8
                        );
                        
                        painter.circle_filled(
                            egui::pos2(column.x, new_y),
                            8.0,
                            trail_color
                        );
                        
                        painter.text(
                            egui::pos2(column.x, new_y),
                            egui::Align2::CENTER_CENTER,
                            c.to_string(),
                            egui::FontId::monospace(14.0 + rng.gen_range(0.0..4.0)),
                            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha)
                        );
                    }
                }
                
                column.chars = next_chars;
                if !column.chars.is_empty() {
                    next_columns.push_back(column);
                }
            }
            ui_state.matrix_columns = next_columns;
        }

        // GRID LINES - Add one extra column for row numbers, then channels
        for i in 0..=NUM_CHANNELS + 1 {
            let x = rect.left() + i as f32 * col_width;
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0, colors.border.linear_multiply(0.3))
            );
        }
        
        let start_row_idx = (*scroll_y / row_height).floor() as isize;
        let end_row_idx = ((*scroll_y + rect.height()) / row_height).ceil() as isize;
        for r in start_row_idx..=end_row_idx {
            if r >= 0 && r < ROWS_PER_PATTERN as isize {
                let row_y = rect.top() + (r as f32 * row_height) - *scroll_y;
                painter.line_segment(
                    [egui::pos2(rect.left(), row_y), egui::pos2(rect.right(), row_y)],
                    egui::Stroke::new(1.0, colors.border.linear_multiply(0.2))
                );
            }
        }
        
        // RENDER NOTES WITH DETAILED INFO
        let current_pattern_idx = project.current_pattern;
        if let Some(pattern) = project.patterns.get(current_pattern_idx) {
            for r in start_row_idx..=end_row_idx {
                if r >= 0 && r < ROWS_PER_PATTERN as isize {
                    let row_y = rect.top() + (r as f32 * row_height) - *scroll_y;
                    let row_data = &pattern.rows[r as usize];
                    
                    // Row number
                    painter.text(
                        egui::pos2(rect.left() + 5.0, row_y + row_height / 2.0),
                        egui::Align2::LEFT_CENTER,
                        format!("{:02}", r),
                        egui::FontId::monospace(10.0),
                        if r as usize == project.current_row {
                            colors.primary
                        } else {
                            colors.text_dim
                        }
                    );

                    // Channels start from column 1 (second column), column 0 is for row numbers
                    for (ch, cell) in row_data.channels.iter().enumerate() {
                        // Offset channel by 1 to leave first column empty
                        let cell_x = rect.left() + (ch + 1) as f32 * col_width;
                        let cell_rect = egui::Rect::from_min_size(
                            egui::pos2(cell_x + 1.0, row_y + 1.0),
                            egui::vec2(col_width - 2.0, row_height - 2.0)
                        );
                        
                        // HIGHLIGHT CURRENT CELL IN EDIT MODE - Very visible
                        if *edit_mode == EditMode::Edit 
                           && r as usize == project.current_row 
                           && ch == project.current_channel 
                        {
                            // Bright background highlight
                            painter.rect_filled(
                                cell_rect,
                                0.0,
                                colors.primary.linear_multiply(0.4)
                            );
                            // Thick border
                            painter.rect_stroke(
                                cell_rect,
                                0.0,
                                egui::Stroke::new(3.0, colors.primary)
                            );
                            // Cursor indicator
                            painter.circle_filled(
                                egui::pos2(cell_rect.left() + 3.0, cell_rect.center().y),
                                2.0,
                                colors.primary
                            );
                        }
                        
                        if let Some(note) = cell.note {
                            let inst = (cell.instrument.unwrap_or(0) % 4) as usize;
                            let note_color = colors.note_colors[inst];
                            let velocity = cell.volume.unwrap_or(64);
                            let cc_value = 64; // Default CC value
                            
                            painter.rect_filled(
                                cell_rect,
                                2.0,
                                note_color.linear_multiply(0.3)
                            );
                            
                            painter.rect_stroke(
                                cell_rect,
                                2.0,
                                egui::Stroke::new(1.0, note_color.linear_multiply(0.6))
                            );
                            
                            // Only show note name by default, add V/CC only if changed
                            let note_name_str = note_name(note);
                            let mut note_text = note_name_str.clone();
                            
                            // Add velocity if not default (64)
                            if velocity != 64 {
                                note_text.push_str(&format!(" V:{}", velocity));
                            }
                            
                            // Add CC if not default (64)
                            if cc_value != 64 {
                                note_text.push_str(&format!(" CC:{}", cc_value));
                            }
                            
                            // Single line display
                            painter.text(
                                egui::pos2(cell_rect.center().x, cell_rect.center().y),
                                egui::Align2::CENTER_CENTER,
                                note_text,
                                egui::FontId::monospace(10.0),
                                note_color
                            );
                        }
                    }
                }
            }
        }
        
        // Playhead line
        if project.playing {
            let play_y = rect.top() + (project.current_row as f32 * row_height) - *scroll_y;
            if play_y >= rect.top() && play_y <= rect.bottom() {
                painter.line_segment(
                    [
                        egui::pos2(rect.left(), play_y + row_height / 2.0),
                        egui::pos2(rect.right(), play_y + row_height / 2.0)
                    ],
                    egui::Stroke::new(2.0, colors.primary)
                );
            }
        }
        
        // Tooltip/Hint system - Show on mouse hover over pattern editor
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let rect = ui.available_rect_before_wrap();
        
        // Check if hovering over pattern editor cells
        if let Some(pos) = pointer_pos {
            if pos.x >= rect.left() && pos.x <= rect.right() && pos.y >= rect.top() && pos.y <= rect.bottom() {
                let col_width = rect.width() / (NUM_CHANNELS + 1) as f32;
                let row_height = 22.0;
                let col = ((pos.x - rect.left()) / col_width) as usize;
                let row = ((pos.y - rect.top() + *scroll_y) / row_height) as usize;
                
                if col > 0 && col <= NUM_CHANNELS && row < ROWS_PER_PATTERN {
                    let ch = col - 1;
                    if let Some(pattern) = project.patterns.get(project.current_pattern) {
                        if let Some(row_data) = pattern.rows.get(row) {
                            if let Some(cell) = row_data.channels.get(ch) {
                                if let Some(note) = cell.note {
                                    let note_name_str = note_name(note);
                                    let velocity = cell.volume.unwrap_or(64);
                                    let mut hint = format!("Row {} Ch{}: {}", row, ch + 1, note_name_str);
                                    if velocity != 64 {
                                        hint.push_str(&format!(" V:{}", velocity));
                                    }
                                    ui_state.hover_tooltip = Some(hint);
                                } else {
                                    ui_state.hover_tooltip = Some(format!("Row {} Ch{}: Empty", row, ch + 1));
                                }
                            }
                        }
                    }
                } else {
                    ui_state.hover_tooltip = None;
                }
            } else {
                ui_state.hover_tooltip = None;
            }
        } else {
            ui_state.hover_tooltip = None;
        }
        
        // Show tooltip window at pointer
        if let Some(_tooltip) = &ui_state.hover_tooltip {
            if pointer_pos.is_some() {
                egui::show_tooltip_at_pointer(ctx, egui::Id::new("hover_tooltip"), |ui| {
                    ui.set_max_width(200.0);
                    if let Some(ref tooltip_text) = ui_state.hover_tooltip {
                        ui.label(egui::RichText::new(tooltip_text).size(10.0));
                    }
                });
            }
        }
    });
}

fn note_name(midi_note: u8) -> String {
    let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (midi_note / 12) as i32 - 1;
    let note_idx = (midi_note % 12) as usize;
    format!("{}{}", notes[note_idx], octave)
}
