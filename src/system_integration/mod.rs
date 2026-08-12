use std::time::Duration;

use crate::{
    app::AppPaths,
    controller::{
        commands::SystemIntegrationCommand, events::SystemIntegrationEvent, state::PlaybackStatus,
    },
    errors::SystemIntegrationError,
};
use crossbeam_channel::{Receiver, Sender, select};
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
    #[must_use]
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
        let hwnd: Option<*mut std::ffi::c_void> = None;

        #[cfg(target_os = "windows")]
        let hwnd = raw_window_handle.and_then(|handle| {
            match handle {
                RawWindowHandle::Win32(h) => Some(h.hwnd.get() as *mut std::ffi::c_void),
                _ => None,
            }
        });

        let config = PlatformConfig {
            hwnd,
            dbus_name: "app.wiremann.wiremann",
            display_name: "Wiremann",
        };

        let media_controls = MediaControls::new(config).inspect_err(|e| {
            eprintln!("[wiremann] MediaControls::new failed: {e}");
        }).ok();

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

    #[allow(clippy::missing_errors_doc)]
    pub fn run(&mut self) -> Result<(), SystemIntegrationError> {
        let (souvlaki_tx, souvlaki_rx) = crossbeam_channel::unbounded();

        if let Some(controls) = &mut self.media_controls {
            if let Err(e) = controls.attach(move |event| {
                souvlaki_tx.send(event).ok();
            }) {
                eprintln!("[wiremann] MediaControls attach failed: {e}");
                return Ok(());
            }
            eprintln!("[wiremann] MediaControls attached successfully");

            loop {
                select! {
                    recv(self.rx) -> msg => {
                        if let Ok(cmd) = msg {
                            let _ = self.handle_commands(cmd);
                        }
                    }
                    recv(souvlaki_rx) -> msg => {
                        if let Ok(cmd) = msg {self.handle_system_events(&cmd);}
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn handle_commands(
        &mut self,
        cmd: SystemIntegrationCommand,
    ) -> Result<(), SystemIntegrationError> {
        if let Some(controls) = &mut self.media_controls {
            let result: Result<(), SystemIntegrationError> = match cmd {
                SystemIntegrationCommand::SetMetadata {
                    title,
                    artist,
                    album,
                    image: _,
                    duration,
                } => {
                    // souvlaki's Windows path masking has a bug with file:/// URIs,
                    // so we skip the cover image for now. Metadata still works.
                    let cover_url: Option<&str> = None;

                    match controls.set_metadata(MediaMetadata {
                        title: Some(title.as_str()),
                        album: if album.is_empty() { None } else { Some(album.as_str()) },
                        artist: if artist.is_empty() { None } else { Some(artist.as_str()) },
                        cover_url: cover_url.as_deref(),
                        duration: Some(Duration::from_secs(duration)),
                    }) {
                        Ok(()) => eprintln!("[wiremann] set_metadata OK"),
                        Err(e) => eprintln!("[wiremann] set_metadata failed: {e}"),
                    }

                    Ok(())
                }
                SystemIntegrationCommand::SetPosition(pos) => {
                    if let Err(e) = controls.set_playback(MediaPlayback::Playing {
                        progress: Some(MediaPosition(pos)),
                    }) {
                        eprintln!("[wiremann] set_playback (position) failed: {e}");
                    }
                    Ok(())
                }
                SystemIntegrationCommand::SetPlaybackStatus(status, pos) => {
                    let playback = match status {
                        PlaybackStatus::Stopped => MediaPlayback::Stopped,
                        PlaybackStatus::Paused => MediaPlayback::Paused {
                            progress: Some(MediaPosition(pos)),
                        },
                        PlaybackStatus::Playing => MediaPlayback::Playing {
                            progress: Some(MediaPosition(pos)),
                        },
                    };
                    if let Err(e) = controls.set_playback(playback) {
                        eprintln!("[wiremann] set_playback (status) failed: {e}");
                    }
                    Ok(())
                }
            };

            if let Err(e) = result {
                eprintln!("[wiremann] handle_commands error: {e}");
            }
        }

        Ok(())
    }

    fn handle_system_events(&mut self, event: &MediaControlEvent) {
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
                        .send(SystemIntegrationEvent::SeekForward(*secs))
                        .ok();
                }
                SeekDirection::Backward => {
                    self.tx
                        .send(SystemIntegrationEvent::SeekBackward(*secs))
                        .ok();
                }
            },
            MediaControlEvent::SetPosition(pos) => {
                self.tx.send(SystemIntegrationEvent::Position(pos.0)).ok();
            }
            MediaControlEvent::SetVolume(vol) => {
                self.tx.send(SystemIntegrationEvent::Volume(*vol)).ok();
            }
            _ => {}
        }
    }

}
