use std::{
    fs::File,
    io::{BufWriter, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    app::AppPaths,
    controller::{
        commands::SystemIntegrationCommand, events::SystemIntegrationEvent, state::PlaybackStatus,
    },
    errors::SystemIntegrationError,
};
use crossbeam_channel::{Receiver, Sender, select};
use garb::bytes::bgra_to_rgb;
use image::{ExtendedColorType, codecs::jpeg::JpegEncoder};
use raw_window_handle::RawWindowHandle;
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
    SeekDirection,
};

pub struct SystemIntegration {
    pub tx: Sender<SystemIntegrationEvent>,
    pub rx: Receiver<SystemIntegrationCommand>,
    app_paths: AppPaths,

    media_controls: Option<MediaControls>,
}

impl SystemIntegration {
    #[allow(unused_variables)]
    pub fn new(
        raw_window_handle: Option<RawWindowHandle>,
        app_paths: AppPaths,
    ) -> (
        Self,
        Sender<SystemIntegrationCommand>,
        Receiver<SystemIntegrationEvent>,
    ) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let hwnd = raw_window_handle.and_then(|handle| {
            let handle = match handle {
                RawWindowHandle::Win32(h) => h,
                _ => unreachable!(),
            };
            Some(handle.hwnd)
        });

        let config = PlatformConfig {
            hwnd,
            dbus_name: "app.wiremann.wiremann",
            display_name: "Wiremann",
        };

        let media_controls = MediaControls::new(config).ok();

        (
            Self {
                tx: event_tx,
                rx: cmd_rx,
                app_paths,
                media_controls,
            },
            cmd_tx,
            event_rx,
        )
    }

    pub fn run(&mut self) -> Result<(), SystemIntegrationError> {
        let (souvlaki_tx, souvlaki_rx) = crossbeam_channel::unbounded();

        if let Some(controls) = &mut self.media_controls {
            controls.attach(move |event| {
                souvlaki_tx.send(event).ok();
            })?;

            loop {
                select! {
                    recv(self.rx) -> msg => {
                        if let Ok(cmd) = msg {self.handle_commands(cmd)?;}
                    }
                    recv(souvlaki_rx) -> msg => {
                        if let Ok(cmd) = msg {self.handle_system_events(cmd)?;}
                    }
                }
            }
        }

        Ok(())
    }

    pub fn handle_commands(
        &mut self,
        cmd: SystemIntegrationCommand,
    ) -> Result<(), SystemIntegrationError> {
        if let Some(controls) = &mut self.media_controls {
            match cmd {
                SystemIntegrationCommand::SetMetadata {
                    title,
                    artist,
                    album,
                    image,
                    duration,
                } => {
                    let version = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    let name = format!("current_album_art_{}.jpg", version);

                    let path = self.app_paths.cache.join(&name);

                    if let Some((width, height, bytes)) = image {
                        let mut rgb = vec![0u8; (width * height * 3) as usize];

                        bgra_to_rgb(&bytes, &mut rgb)?;

                        let tmp_path = path.with_extension("jpg.tmp");

                        {
                            let file = File::create(&tmp_path)?;
                            let mut writer = BufWriter::new(file);

                            let mut encoder = JpegEncoder::new_with_quality(&mut writer, 80);
                            encoder.encode(&rgb, width, height, ExtendedColorType::Rgb8)?;
                            writer.flush()?;
                        }

                        std::fs::rename(&tmp_path, &path)?;
                    }

                    let cover_url = format!("file://{}?v={}", path.display(), version);

                    controls.set_metadata(MediaMetadata {
                        title: Some(title.as_str()),
                        album: Some(album.as_str()),
                        artist: Some(artist.as_str()),
                        cover_url: Some(&cover_url),
                        duration: Some(Duration::from_secs(duration)),
                    })?;

                    self.cleanup_images(&name)?;
                }
                SystemIntegrationCommand::SetPosition(pos) => {
                    controls.set_playback(MediaPlayback::Playing {
                        progress: Some(MediaPosition(Duration::from_secs(pos))),
                    })?;
                }
                SystemIntegrationCommand::SetPlaybackStatus(status, pos) => {
                    let status = match status {
                        PlaybackStatus::Stopped => MediaPlayback::Stopped,
                        PlaybackStatus::Paused => MediaPlayback::Paused {
                            progress: Some(MediaPosition(Duration::from_secs(pos))),
                        },
                        PlaybackStatus::Playing => MediaPlayback::Playing {
                            progress: Some(MediaPosition(Duration::from_secs(pos))),
                        },
                    };
                    controls.set_playback(status)?;
                }
            }
        }

        Ok(())
    }

    fn handle_system_events(
        &mut self,
        event: MediaControlEvent,
    ) -> Result<(), SystemIntegrationError> {
        match event {
            MediaControlEvent::Play => {
                self.tx.send(SystemIntegrationEvent::Play).ok();
            }
            MediaControlEvent::Pause => {
                self.tx.send(SystemIntegrationEvent::Pause).ok();
            }
            MediaControlEvent::Toggle => {
                self.tx.send(SystemIntegrationEvent::PlayPause).ok();
            }
            MediaControlEvent::Stop => {
                self.tx.send(SystemIntegrationEvent::Stop).ok();
            }
            MediaControlEvent::Next => {
                self.tx.send(SystemIntegrationEvent::Next).ok();
            }
            MediaControlEvent::Previous => {
                self.tx.send(SystemIntegrationEvent::Prev).ok();
            }
            MediaControlEvent::SeekBy(direction, secs) => match direction {
                SeekDirection::Forward => {
                    self.tx
                        .send(SystemIntegrationEvent::SeekForward(secs.as_secs()))
                        .ok();
                }
                SeekDirection::Backward => {
                    self.tx
                        .send(SystemIntegrationEvent::SeekBackward(secs.as_secs()))
                        .ok();
                }
            },
            MediaControlEvent::SetPosition(pos) => {
                self.tx
                    .send(SystemIntegrationEvent::Position(pos.0.as_secs()))
                    .ok();
            }
            MediaControlEvent::SetVolume(vol) => {
                self.tx.send(SystemIntegrationEvent::Volume(vol)).ok();
            }
            _ => {}
        }
        Ok(())
    }

    fn cleanup_images(&self, current_name: &str) -> std::io::Result<()> {
        let path = &self.app_paths.cache;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("current_album_art_")
                    && name.ends_with(".jpg")
                    && name != current_name
                {
                    let _ = std::fs::remove_file(path);
                }
            }
        }

        Ok(())
    }
}
