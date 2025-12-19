use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use midir::{MidiInput, Ignore};

// --- API ---

#[derive(Event, Debug, Clone)]
pub struct MidiInputEvent {
    pub note: u8,
    pub velocity: u8,
    pub status: u8, // 144 = On, 128 = Off
}

pub enum AudioCommand {
    PlayNote {
        note: u8,
        #[allow(dead_code)]
        instrument: u8,
        velocity: u8,
        channel: usize,
    },
    #[allow(dead_code)]
    StopNote {
        channel: usize,
    },
    #[allow(dead_code)]
    SetBpm(f32),
}

#[derive(Resource)]
pub struct AudioEngine {
    pub sender: Sender<AudioCommand>,
}

#[derive(Resource)]
struct MidiReceiver(Receiver<MidiInputEvent>);

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MidiInputEvent>()
           .add_systems(Startup, setup_audio)
           .add_systems(Update, dispatch_midi_events);
    }
}

// --- Implementation ---

const SAMPLE_RATE: f32 = 44100.0;
const NUM_CHANNELS: usize = 8;

struct Voice {
    active: bool,
    note: u8,
    phase: f32,
    envelope: f32,
    instrument: u8, // Store instrument type per voice
    #[allow(dead_code)]
    decay: f32,
}

impl Default for Voice {
    fn default() -> Self {
        Self { 
            active: false, 
            note: 0, 
            phase: 0.0, 
            envelope: 0.0, 
            instrument: 0,
            decay: 0.9999 
        }
    }
}

struct AudioState {
    voices: [Voice; NUM_CHANNELS],
    bpm: f32,
}

fn setup_audio(mut commands: Commands) {
    // 1. Channels
    let (audio_tx, audio_rx) = unbounded::<AudioCommand>();
    let (midi_tx, midi_rx) = unbounded::<MidiInputEvent>();

    commands.insert_resource(AudioEngine { sender: audio_tx });
    commands.insert_resource(MidiReceiver(midi_rx));

    // 2. Audio Thread (CPAL)
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("ERROR: No default audio output device found!");
                return;
            }
        };

        let config = match device.default_output_config() {
            Ok(c) => c.into(),
            Err(e) => {
                eprintln!("ERROR: Failed to get audio config: {}", e);
                return;
            }
        };

        let state = Arc::new(Mutex::new(AudioState {
            voices: [
                Voice::default(), Voice::default(), Voice::default(), Voice::default(),
                Voice::default(), Voice::default(), Voice::default(), Voice::default()
            ],
            bpm: 120.0,
        }));

        let state_cb = state.clone();
        
        let stream_result = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut state = state_cb.lock().unwrap();
                
                // Process Commands
                while let Ok(cmd) = audio_rx.try_recv() {
                    match cmd {
                        AudioCommand::PlayNote { note, channel, velocity, instrument } => {
                            if channel < NUM_CHANNELS {
                                state.voices[channel].active = true;
                                state.voices[channel].note = note;
                                state.voices[channel].phase = 0.0;
                                state.voices[channel].envelope = (velocity as f32 / 127.0) * 0.5;
                                state.voices[channel].instrument = instrument;
                            }
                        },
                        AudioCommand::StopNote { channel } => {
                            if channel < NUM_CHANNELS {
                                state.voices[channel].active = false;
                            }
                        },
                        AudioCommand::SetBpm(bpm) => {
                            state.bpm = bpm;
                        },
                    }
                }

                // Render Audio
                for sample in data.iter_mut() {
                    let mut mixed = 0.0;
                    
                    for voice in state.voices.iter_mut() {
                        if voice.active {
                            let freq = 440.0 * 2.0_f32.powf((voice.note as f32 - 69.0) / 12.0);
                            let value = match voice.instrument % 4 {
                                0 => (voice.phase * 2.0 * std::f32::consts::PI).sin(), // Sine wave
                                1 => if voice.phase < 0.5 { 1.0 } else { -1.0 }, // Square wave
                                2 => (voice.phase * 2.0 - 1.0) * 2.0, // Triangle/Sawtooth
                                _ => {
                                    // Pulse wave with variable width
                                    let pulse_width = 0.3;
                                    if voice.phase < pulse_width { 1.0 } else { -0.3 }
                                },
                            };
                            
                            mixed += value * voice.envelope;
                            
                            voice.phase += freq / SAMPLE_RATE;
                            if voice.phase > 1.0 { voice.phase -= 1.0; }
                            
                            voice.envelope *= 0.99995; 
                            if voice.envelope < 0.001 { voice.active = false; }
                        }
                    }
                    *sample = mixed.clamp(-1.0, 1.0); 
                }
            },
            |err| eprintln!("Audio Stream Error: {}", err),
            None,
        );

        if let Ok(stream) = stream_result {
            if let Err(e) = stream.play() {
                eprintln!("ERROR: Failed to play audio stream: {}", e);
            }
            std::thread::park(); // Keep audio thread alive
        } else {
             eprintln!("ERROR: Failed to build audio stream: {}", stream_result.err().unwrap());
        }
    });

    // 3. MIDI Thread (Midir)
    std::thread::spawn(move || {
        let midi_in_result = MidiInput::new("Terminal Velocity Input");
        match midi_in_result {
            Ok(mut midi_in) => {
                midi_in.ignore(Ignore::None);
                
                let ports = midi_in.ports();
                if let Some(port) = ports.get(0) {
                    println!("Connecting to MIDI port: {}", midi_in.port_name(port).unwrap_or_default());
                    
                    let _conn = midi_in.connect(port, "tv-input-conn", move |_, message, _| {
                        if message.len() >= 3 {
                             let status = message[0] & 0xF0;
                             if status == 0x90 || status == 0x80 {
                                  let _ = midi_tx.send(MidiInputEvent {
                                      status,
                                      note: message[1],
                                      velocity: message[2],
                                  });
                             }
                        }
                    }, ());
                    
                    // Keep connection alive
                    loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
                } else {
                    println!("No MIDI ports found.");
                }
            },
            Err(e) => {
                eprintln!("ERROR: Failed to initialize MIDI Input: {}", e);
            }
        }
    });
}

fn dispatch_midi_events(
    mut events: EventWriter<MidiInputEvent>,
    receiver: Res<MidiReceiver>,
) {
    while let Ok(event) = receiver.0.try_recv() {
        events.send(event);
    }
}
