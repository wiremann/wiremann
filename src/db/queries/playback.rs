use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Query, Expr, SqliteQueryBuilder, IntoColumnRef};
use sea_query_rusqlite::RusqliteBinder;

use crate::controller::state::{self, PlaybackState, PlaybackStatus, PlaylistId, TrackId};
use crate::db::tables::Playbacks;

pub fn get_playback_state(conn: &Connection) -> Result<PlaybackState> {
    let query = Query::select()
        .expr(Expr::col((Playbacks::Table, Playbacks::CurrentTrack)))
        .expr(Expr::col((Playbacks::Table, Playbacks::CurrentPlaylist)))
        .expr(Expr::col((Playbacks::Table, Playbacks::CurrentIndex)))
        .expr(Expr::col((Playbacks::Table, Playbacks::Status)))
        .expr(Expr::col((Playbacks::Table, Playbacks::Position)))
        .expr(Expr::col((Playbacks::Table, Playbacks::Volume)))
        .expr(Expr::col((Playbacks::Table, Playbacks::Mute)))
        .expr(Expr::col((Playbacks::Table, Playbacks::Shuffling)))
        .expr(Expr::col((Playbacks::Table, Playbacks::Repeat)))
        .from(Playbacks::Table)
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();

    let mut stmt = conn.prepare(&sql)?;

    let mut rows = stmt.query_map(params.as_slice(), |row| {
        let current_track: Option<Vec<u8>> = row.get(0)?;
        let current_playlist: Option<String> = row.get(1)?;
        let current_index: i64 = row.get(2)?;
        let status: String = row.get(3)?;
        let position: i64 = row.get(4)?;
        let volume: f64 = row.get(5)?;
        let mute: bool = row.get(6)?;
        let shuffling: bool = row.get(7)?;
        let repeat: bool = row.get(8)?;

        Ok((current_track, current_playlist, current_index, status, position, volume, mute, shuffling, repeat))
    })?;

    if let Some(res) = rows.next() {
        let (current_track, current_playlist, current_index, status, position, volume, mute, shuffling, repeat) = res?;

        let current = current_track.and_then(|b| {
            if b.len() == 16 {
                let arr: [u8;16] = b.try_into().ok()?;
                Some(TrackId(arr))
            } else {
                None
            }
        });

        let current_playlist = current_playlist.and_then(|s| uuid::Uuid::parse_str(&s).ok().map(PlaylistId));

        let status = match status.as_str() {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        };

        Ok(PlaybackState {
            current,
            current_playlist,
            current_index: current_index as usize,
            status,
            position: std::time::Duration::from_millis(position as u64),
            volume: volume as f32,
            mute,
            shuffling,
            repeat,
        })
    } else {
        Ok(PlaybackState::default())
    }
}

pub fn save_playback_state(conn: &mut Connection, state: &PlaybackState) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM playbacks", [])?;

    let current_track: Option<Vec<u8>> = state.current.map(|t| t.0.to_vec());
    let current_playlist: Option<String> = state.current_playlist.map(|p| p.0.to_string());
    let current_index: i64 = state.current_index as i64;
    let status: &str = match state.status {
        PlaybackStatus::Playing => "Playing",
        PlaybackStatus::Paused => "Paused",
        PlaybackStatus::Stopped => "Stopped",
    };
    let position: i64 = state.position.as_millis() as i64;
    let volume: f64 = state.volume as f64;
    let mute: bool = state.mute;
    let shuffling: bool = state.shuffling;
    let repeat: bool = state.repeat;

    let query = sea_query::Query::insert()
        .into_table(Playbacks::Table)
        .columns([
            Playbacks::Id,
            Playbacks::CurrentTrack,
            Playbacks::CurrentPlaylist,
            Playbacks::CurrentIndex,
            Playbacks::Status,
            Playbacks::Position,
            Playbacks::Volume,
            Playbacks::Mute,
            Playbacks::Shuffling,
            Playbacks::Repeat,
        ])
        .values_panic(vec![
            1u32.into(),
            current_track.into(),
            current_playlist.into(),
            current_index.into(),
            status.into(),
            position.into(),
            volume.into(),
            mute.into(),
            shuffling.into(),
            repeat.into(),
        ])
        .to_owned();

    let (sql, values) = query.build_rusqlite(sea_query::SqliteQueryBuilder);
    let params = values.as_params();

    tx.execute(&sql, params.as_slice())?;

    tx.commit()?;

    Ok(())
}
