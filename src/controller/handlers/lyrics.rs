use super::{App, CacherCommand, Controller, ControllerError, Entity, LyricsEvent, Wiremann};
use crate::ui::pages::player::lyrics::{LyricsState, LyricsStatus};

impl Controller {
    pub fn handle_lyrics_event(
        &mut self,
        cx: &mut App,
        event: &LyricsEvent,
        _view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            LyricsEvent::Lyrics(id, lyrics) => {
                let current = cx.global::<Controller>().state.read(cx).playback.current;

                if let Some(current) = current
                    && current == *id
                {
                    let lyrics_state = cx.global::<LyricsState>().0.clone();

                    lyrics_state.update(cx, |this, cx| {
                        this.lyrics.clone_from(lyrics);
                        this.track_id = Some(current);

                        this.status = if lyrics.is_some() {
                            LyricsStatus::Available
                        } else {
                            LyricsStatus::Unavailable
                        };

                        cx.notify();
                    });
                }
                if let Some(lyrics) = lyrics {
                    self.cacher_tx
                        .send(CacherCommand::WriteLyrics(*id, lyrics.clone()))
                        .ok();
                }
            }
        }

        Ok(())
    }
}
