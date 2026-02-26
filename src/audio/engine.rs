use crossbeam_channel::{Receiver, Sender};
use std::time::Duration;
use std::{fs::File, path::PathBuf};

use crate::controller::state::PlaybackStatus;
use crate::{
    controller::{commands::AudioCommand, events::AudioEvent},
    errors::AudioError,
};
use rodio::{decoder::DecoderBuilder, OutputStream, OutputStreamBuilder, Sink};

pub struct Audio {
    sink: Sink,
    stream_handle: OutputStream,

    pub rx: Receiver<AudioCommand>,
    pub tx: Sender<AudioEvent>,

    track_ended: bool,
}

impl Audio {
    pub fn new() -> (Self, Sender<AudioCommand>, Receiver<AudioEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
        let sink = Sink::connect_new(&stream_handle.mixer());

        let engine = Audio {
            stream_handle,
            sink,
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
                    AudioCommand::Load(path) => self.load_path(PathBuf::from(path))?,
                    AudioCommand::GetPosition => self.emit_position()?,
                    AudioCommand::CheckTrackEnded => self.check_track_ended()?,
                    AudioCommand::Play => self.play()?,
                    AudioCommand::Pause => self.pause()?,
                    AudioCommand::Stop => self.stop()?,
                    AudioCommand::SetVolume(v) => self.set_volume(v)?,
                    AudioCommand::Seek(u64) => self.seek(u64)?,
                }
            }
        }
    }

    fn load_path(&mut self, path: PathBuf) -> Result<(), AudioError> {
        self.sink.stop();
        
        let prev_vol = self.sink.volume();
        
        self.sink = Sink::connect_new(self.stream_handle.mixer());

        let file = File::open(path.clone()).unwrap();
        let len = file.metadata().unwrap().len();
        let source = DecoderBuilder::new()
            .with_data(file)
            .with_byte_len(len)
            .with_seekable(true)
            .build()
            .unwrap();

        self.sink.append(source);
        
        self.sink.set_volume(prev_vol);

        let _ = self.tx.send(AudioEvent::TrackLoaded(path));

        self.play()?;

        Ok(())
    }

    fn emit_position(&self) -> Result<(), AudioError> {
        let _ = self
            .tx
            .send(AudioEvent::Position(self.sink.get_pos().as_secs()));
        Ok(())
    }

    fn play(&self) -> Result<(), AudioError> {
        self.sink.play();
        let _ = self
            .tx
            .send(AudioEvent::PlaybackStatus(PlaybackStatus::Playing));

        Ok(())
    }

    fn pause(&self) -> Result<(), AudioError> {
        self.sink.pause();
        let _ = self
            .tx
            .send(AudioEvent::PlaybackStatus(PlaybackStatus::Paused));

        Ok(())
    }

    fn stop(&self) -> Result<(), AudioError> {
        self.sink.stop();
        let _ = self
            .tx
            .send(AudioEvent::PlaybackStatus(PlaybackStatus::Stopped));

        Ok(())
    }

    fn set_volume(&self, volume: f32) -> Result<(), AudioError> {
        let volume = volume.clamp(0.0, 1.0);
        self.sink.set_volume(volume);
        let _ = self.tx.send(AudioEvent::Volume(volume));

        Ok(())
    }

    fn seek(&self, pos: u64) -> Result<(), AudioError> {
        self.sink.try_seek(Duration::from_secs(pos))?;

        Ok(())
    }

    fn check_track_ended(&mut self) -> Result<(), AudioError> {
        if self.sink.is_paused() {
            return Ok(());
        }

        if self.sink.empty() && !self.track_ended {
            self.track_ended = true;

            let _ = self.tx.send(AudioEvent::TrackEnded);
        }

        Ok(())
    }
}
