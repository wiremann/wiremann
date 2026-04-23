use crate::{
    app::AppPaths,
    controller::{commands::LyricsCommand, events::LyricsEvent},
    errors::LyricsError,
};
use crossbeam_channel::{Receiver, Sender, select};

pub struct Lyrics {
    pub tx: Sender<LyricsEvent>,
    pub rx: Receiver<LyricsCommand>,
    app_paths: AppPaths,
}

impl Lyrics {
    #[allow(unused_variables)]
    #[must_use]
    pub fn new(app_paths: AppPaths) -> (Self, Sender<LyricsCommand>, Receiver<LyricsEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        (
            Self {
                tx: event_tx,
                rx: cmd_rx,
                app_paths,
            },
            cmd_tx,
            event_rx,
        )
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(&mut self) -> Result<(), LyricsError> {
        loop {
            select! {
                recv(self.rx) -> msg => {
                    if let Ok(cmd) = msg {self.handle_commands(cmd)?;}
                }
            }
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn handle_commands(&mut self, cmd: LyricsCommand) -> Result<(), LyricsError> {
        match cmd {
            _ => {}
        }

        Ok(())
    }
}
