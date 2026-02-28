use crate::controller::player::ScannerCommand;
use crate::controller::{
    metadata::Metadata,
    player::{AudioCommand, AudioEvent, PlayerState},
};
use crate::scanner::cache::AppStateCache;
use crate::scanner::ScannerState;
use crate::utils::decode_thumbnail;
use crossbeam_channel::{select, tick, Receiver, Sender};
use rand::prelude::SliceRandom;
use rodio::{decoder::DecoderBuilder, OutputStream, OutputStreamBuilder, Sink};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf, time::Duration};

pub struct AudioEngine {
    sink: Sink,
    stream_handle: OutputStream,
    player_state: PlayerState,
    audio_cmd_rx: Receiver<AudioCommand>,
    scanner_state: ScannerState,
    audio_event_tx: Sender<AudioEvent>,
    track_ended: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

impl AudioEngine {
    pub fn run(audio_cmd_rx: Receiver<AudioCommand>, audio_event_tx: Sender<AudioEvent>) {
        let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
        let sink = Sink::connect_new(&stream_handle.mixer());

        let mut engine = AudioEngine {
            sink,
            stream_handle,
            player_state: PlayerState::default(),
            scanner_state: ScannerState::default(),
            audio_cmd_rx,
            audio_event_tx,
            track_ended: false,
        };

        engine.event_loop();
    }

    fn event_loop(&mut self) {
        let ticker = tick(Duration::from_millis(500));

        loop {
            select! {
                recv(self.audio_cmd_rx) -> msg => {
                    let cmd = match msg {
                        Ok(c) => c,
                        Err(_) => break,
                    };

                    match cmd {
                        AudioCommand::Load(path) => self.load_path(PathBuf::from(path)),
                        AudioCommand::LoadId(id) => self.load_at(id),
                        AudioCommand::Play => self.play(),
                        AudioCommand::Pause => self.pause(),
                        AudioCommand::Stop => self.stop(),
                        AudioCommand::Volume(vol) => self.set_volume(vol),
                        AudioCommand::Seek(pos) => self.seek(pos),
                        AudioCommand::Mute => self.set_mute(),
                        AudioCommand::ScannerState(scanner_state) => self.scanner_state(scanner_state),
                        AudioCommand::Next => self.next(),
                        AudioCommand::Prev => self.prev(),
                        AudioCommand::Repeat => self.set_repeat(),
                        AudioCommand::Shuffle => self.set_shuffle(),
                        AudioCommand::SetAppState{app_state_cache, scanner_cmd_tx} => self.set_app_state(app_state_cache, scanner_cmd_tx),
                    }
                }

                recv(ticker) -> _ => {
                    self.emit_position();
                    self.check_track_end();
                }
            }
        }
    }

    fn load(&mut self, path: PathBuf) {
        self.sink.stop();
        self.sink = Sink::connect_new(self.stream_handle.mixer());
        self.track_ended = false;

        self.player_state.current = Some(path.clone());

        let file = File::open(path.clone()).unwrap();
        let len = file.metadata().unwrap().len();
        let source = DecoderBuilder::new()
            .with_data(file)
            .with_byte_len(len)
            .with_seekable(true)
            .build()
            .unwrap();

        if self.player_state.mute {
            self.sink.set_volume(0.0);
        } else {
            self.sink.set_volume(self.player_state.volume);
        }
        self.sink.append(source);

        let _ = self
            .audio_event_tx
            .send(AudioEvent::TrackLoaded(path.clone()));

        self.player_state.state = PlaybackState::Playing;

        self.meta(Metadata::read(path).expect("No metadata"));

        let _ = self
            .audio_event_tx
            .send(AudioEvent::PlayerStateChanged(self.player_state.clone()));
    }

    pub fn load_path(&mut self, path: PathBuf) {
        if let (Some(playlist), Some(queue)) = (
            &self.scanner_state.current_playlist,
            Some(&self.scanner_state.queue_order),
        ) {
            if let Some(real_index) = playlist.tracks.iter().position(|t| t.path == path) {
                if let Some(queue_pos) = queue.iter().position(|&i| i == real_index) {
                    self.player_state.index = queue_pos;
                }
            }
        }

        self.load(path);
    }

    pub fn load_at(&mut self, queue_index: usize) {
        self.player_state.index = queue_index;

        let playlist = self.scanner_state.current_playlist.as_ref().unwrap();
        let real_index = self.scanner_state.queue_order[queue_index];
        let track = &playlist.tracks[real_index];

        self.load(track.path.clone());
    }

    fn meta(&mut self, meta: Metadata) {
        if let Some(data) = meta.thumbnail.clone() {
            match decode_thumbnail(data.into_boxed_slice(), false) {
                Ok(thumbnail) => self.player_state.thumbnail = Some(thumbnail),
                Err(_) => {}
            }
        }
        self.player_state.meta = Some(meta);
        self.send_player_state();
    }

    fn scanner_state(&mut self, scanner_state: ScannerState) {
        self.scanner_state = scanner_state;
        self.send_scanner_state();
    }

    fn play(&mut self) {
        if self.player_state.state != PlaybackState::Playing {
            self.sink.play();
            self.player_state.state = PlaybackState::Playing;
            let _ = self
                .audio_event_tx
                .send(AudioEvent::PlayerStateChanged(self.player_state.clone()));
        }
    }

    fn pause(&mut self) {
        if self.player_state.state == PlaybackState::Playing {
            self.sink.pause();
            self.player_state.state = PlaybackState::Paused;
            let _ = self
                .audio_event_tx
                .send(AudioEvent::PlayerStateChanged(self.player_state.clone()));
        }
    }

    fn stop(&mut self) {
        self.sink.stop();
        self.player_state.state = PlaybackState::Stopped;
        let _ = self
            .audio_event_tx
            .send(AudioEvent::PlayerStateChanged(self.player_state.clone()));
    }

    fn set_volume(&mut self, volume: f32) {
        self.player_state.volume = volume.clamp(0.0, 1.0);
        self.sink.set_volume(self.player_state.volume);

        let _ = self
            .audio_event_tx
            .send(AudioEvent::PlayerStateChanged(self.player_state.clone()));
    }

    fn set_mute(&mut self) {
        self.player_state.mute = !self.player_state.mute;
        if self.player_state.mute {
            self.sink.set_volume(0.0);
        } else {
            self.sink.set_volume(self.player_state.volume);
        }
        let _ = self
            .audio_event_tx
            .send(AudioEvent::PlayerStateChanged(self.player_state.clone()));
    }

    fn send_player_state(&mut self) {
        let _ = self
            .audio_event_tx
            .send(AudioEvent::PlayerStateChanged(self.player_state.clone()));
    }

    fn send_scanner_state(&mut self) {
        let _ = self
            .audio_event_tx
            .send(AudioEvent::ScannerStateChanged(self.scanner_state.clone()));
    }

    fn emit_position(&mut self) {
        if self.player_state.state == PlaybackState::Playing {
            self.player_state.position = self.sink.get_pos().as_secs();
            self.send_player_state();
        }
    }

    fn seek(&mut self, pos: u64) {
        self.sink.try_seek(Duration::from_secs(pos)).unwrap();
    }

    fn next(&mut self) {
        if self.scanner_state.current_playlist.is_none() {
            return;
        }

        let len = self.scanner_state.queue_order.len();
        if len == 0 {
            return;
        }
        self.player_state.index = (self.player_state.index + 1) % len;

        self.load_at(self.player_state.index);
    }

    fn prev(&mut self) {
        if self.scanner_state.current_playlist.is_none() {
            return;
        }

        let len = self.scanner_state.queue_order.len();
        if len == 0 {
            return;
        }

        self.player_state.index = if self.player_state.index == 0 {
            len - 1
        } else {
            self.player_state.index - 1
        };

        self.load_at(self.player_state.index);
    }

    fn check_track_end(&mut self) {
        if self.player_state.state != PlaybackState::Playing {
            return;
        }

        if self.sink.empty() && !self.track_ended {
            self.track_ended = true;

            let _ = self.audio_event_tx.send(AudioEvent::TrackEnded);
        }
    }

    fn set_repeat(&mut self) {
        self.player_state.repeat = !self.player_state.repeat;
        self.send_player_state();
    }

    fn set_shuffle(&mut self) {
        self.player_state.shuffling = !self.player_state.shuffling;
        if self.player_state.shuffling {
            let playlist = match &self.scanner_state.current_playlist {
                Some(p) => p,
                None => return,
            };

            let len = playlist.tracks.len();

            self.scanner_state.queue_order = (0..len).collect();
            self.scanner_state.queue_order.shuffle(&mut rand::rng());

            let current_actual_index = self.player_state.index;

            // Start from current song
            if let Some(pos) = self
                .scanner_state
                .queue_order
                .iter()
                .position(|&i| i == current_actual_index)
            {
                self.scanner_state.queue_order.swap(0, pos);
                self.player_state.index = 0;
            }
        } else {
            let playlist = match &self.scanner_state.current_playlist {
                Some(p) => p,
                None => return,
            };

            let len = playlist.tracks.len();

            let real_index = self.scanner_state.queue_order[self.player_state.index];

            self.scanner_state.queue_order = (0..len).collect();
            self.player_state.index = real_index;
        }
        self.send_player_state();
        self.send_scanner_state();
    }

    fn set_app_state(&mut self, app_state_cache: AppStateCache, scanner_cmd_tx: Sender<ScannerCommand>) {
        let _ = scanner_cmd_tx.send(ScannerCommand::Load(app_state_cache.playlist));

        self.scanner_state.queue_order = app_state_cache.queue_order;

        // self.load_at(app_state_cache.index);

        match app_state_cache.state {
            PlaybackState::Playing => self.play(),
            PlaybackState::Paused => self.pause(),
            PlaybackState::Stopped => {}
        }

        self.seek(app_state_cache.position);
        self.set_volume(app_state_cache.volume);
        if app_state_cache.mute {
            self.set_mute()
        }
        if app_state_cache.shuffling {
            self.player_state.shuffling = true
        }
        if app_state_cache.repeat {
            self.set_repeat()
        }
    }
}
