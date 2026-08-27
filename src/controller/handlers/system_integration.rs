use super::{
    App, Controller, ControllerError, Entity, PlaybackStatus, SystemIntegrationEvent, Wiremann,
};

impl Controller {
    pub fn handle_system_integration_event(
        &mut self,
        cx: &mut App,
        event: &SystemIntegrationEvent,
        _view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            SystemIntegrationEvent::PlayPause => {
                let status = self.state.read(cx).playback.status;

                if status == PlaybackStatus::Stopped || status == PlaybackStatus::Paused {
                    self.play();
                } else {
                    self.pause();
                }
            }
            SystemIntegrationEvent::Play => {
                self.play();
            }
            SystemIntegrationEvent::Pause => {
                self.pause();
            }
            SystemIntegrationEvent::Stop => {
                self.stop();
            }
            SystemIntegrationEvent::Next => {
                self.next(cx);
            }
            SystemIntegrationEvent::Prev => {
                self.prev(cx);
            }
            SystemIntegrationEvent::SeekForward(duration) => {
                let pos = self.state.read(cx).playback.position;

                self.seek(pos.saturating_add(*duration));
            }
            SystemIntegrationEvent::SeekBackward(duration) => {
                let pos = self.state.read(cx).playback.position;

                self.seek(pos.saturating_sub(*duration));
            }
            SystemIntegrationEvent::Volume(vol) => {
                #[allow(clippy::cast_possible_truncation)]
                self.set_volume(*vol as f32, cx);
            }
            SystemIntegrationEvent::Position(pos) => {
                self.seek(*pos);
            }
        }

        Ok(())
    }
}
