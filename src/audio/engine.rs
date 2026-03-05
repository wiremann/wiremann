use crossbeam_channel::{Receiver, Sender};
use std::time::Duration;
use std::{fs::File, path::PathBuf};

use crate::controller::state::PlaybackStatus;
use crate::{
    controller::{commands::AudioCommand, events::AudioEvent},
    errors::AudioError,
};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, decoder::DecoderBuilder};

pub struct Audio {
    player: Player,
    stream_handle: MixerDeviceSink,

    pub rx: Receiver<AudioCommand>,
    pub tx: Sender<AudioEvent>,

    track_ended: bool,
}

impl Audio {
    pub fn new() -> (Self, Sender<AudioCommand>, Receiver<AudioEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let stream_handle = DeviceSinkBuilder::open_default_sink().unwrap();
        let player = Player::connect_new(stream_handle.mixer());

        let engine = Audio {
            stream_handle,
            player,
            rx: cmd_rx,
            tx: event_tx,
            track_ended: false,
        };

        (engine, cmd_tx, event_rx)
    }

    pub fn run(&mut self) -> Result<(), AudioError> {
        loop {
            while let Ok(cmd) = self.rx.try_recv() {
                match cmd {
                    AudioCommand::Load(path) => self.load_path(path)?,
                    AudioCommand::GetPosition => self.emit_position(),
                    AudioCommand::CheckTrackEnded => self.check_track_ended(),
                    AudioCommand::Play => self.play(),
                    AudioCommand::Pause => self.pause(),
                    AudioCommand::Stop => self.stop(),
                    AudioCommand::SetVolume(v) => self.set_volume(v),
                    AudioCommand::Seek(u64) => self.seek(u64)?,
                }
            }
        }
    }

    fn load_path(&mut self, path: PathBuf) -> Result<(), AudioError> {
        self.player.stop();

        let prev_vol = self.player.volume();

        self.player = Player::connect_new(self.stream_handle.mixer());

        let file = File::open(path.clone())?;
        let len = file.metadata()?.len();
        let source = DecoderBuilder::new()
            .with_data(file)
            .with_byte_len(len)
            .with_seekable(true)
            .build()?;

        self.player.append(source);

        self.track_ended = false;

        self.player.set_volume(prev_vol);

        let _ = self.tx.send(AudioEvent::TrackLoaded(path));

        self.play();

        Ok(())
    }

    fn emit_position(&self) {
        let _ = self
            .tx
            .send(AudioEvent::Position(self.player.get_pos().as_secs()));
    }

    fn play(&self) {
        self.player.play();
        let _ = self
            .tx
            .send(AudioEvent::PlaybackStatus(PlaybackStatus::Playing));
    }

    fn pause(&self) {
        self.player.pause();
        let _ = self
            .tx
            .send(AudioEvent::PlaybackStatus(PlaybackStatus::Paused));
    }

    fn stop(&self) {
        self.player.stop();
        let _ = self
            .tx
            .send(AudioEvent::PlaybackStatus(PlaybackStatus::Stopped));
    }

    fn set_volume(&self, volume: f32){
        let volume = volume.clamp(0.0, 1.0);
        self.player.set_volume(volume);
    }

    fn seek(&self, pos: u64) -> Result<(), AudioError> {
        self.player.try_seek(Duration::from_secs(pos))?;

        Ok(())
    }

    fn check_track_ended(&mut self) {
        if self.player.empty() && !self.track_ended {
            self.track_ended = true;

            let _ = self.tx.send(AudioEvent::TrackEnded);
        }
    }
}
