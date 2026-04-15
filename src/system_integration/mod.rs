use crate::controller::{commands::SystemIntegrationCommand, events::SystemIntegrationEvent};
use crossbeam_channel::{Receiver, Sender};
use raw_window_handle::RawWindowHandle;
use souvlaki::{MediaControls, PlatformConfig};

pub struct SystemIntegration {
    pub tx: Sender<SystemIntegrationEvent>,
    pub rx: Receiver<SystemIntegrationCommand>,

    media_controls: Option<MediaControls>,
}

impl SystemIntegration {
    pub fn new(raw_window_handle: Option<RawWindowHandle>) -> (
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
                _ => unreachable!()
            };
            Some(handle.hwnd)
        });

        let config = PlatformConfig {
            hwnd,
            dbus_name: "app.wiremann.wiremann",
            display_name: "Wiremann"
        };

        let controls = MediaControls::new(config).ok();

        (
            Self {
                tx: event_tx,
                rx: cmd_rx,
                media_controls: None,
            },
            cmd_tx,
            event_rx,
        )
    }

    pub fn run(self) {

        let (souvlaki_tx, souvlaki_rx) = crossbeam_channel::unbounded();
    }
}
