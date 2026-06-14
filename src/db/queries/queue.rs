use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Query, Expr, SqliteQueryBuilder, IntoColumnRef};
use sea_query_rusqlite::RusqliteBinder;

use crate::controller::state::{self, TrackId, QueueState};
use crate::db::tables::Queue;

pub fn get_queue(conn: &Connection) -> Result<QueueState> {
    let query = Query::select()
        .expr(Expr::col((Queue::Table, Queue::Position)))
        .expr(Expr::col((Queue::Table, Queue::TrackHash)))
        .from(Queue::Table)
        .order_by((Queue::Table, Queue::Position), sea_query::Order::Asc)
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();

    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| {
        let pos: i64 = row.get(0)?;
        let hash: Vec<u8> = row.get(1)?;
        Ok((pos, hash))
    })?;

    let mut tracks = Vec::new();
    let mut order = Vec::new();

    for row in rows {
        let (pos, hash) = row?;
        if hash.len() == 16 {
            let arr: [u8;16] = hash.try_into().unwrap();
            tracks.push(TrackId(arr));
            order.push(tracks.len() - 1);
        }
    }

    Ok(QueueState { tracks, order })
}

pub fn save_queue(conn: &mut Connection, queue: &QueueState) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM queue", [])?;

    for (i, id) in queue.tracks.iter().enumerate() {
        let query = sea_query::Query::insert()
            .into_table(Queue::Table)
            .columns([Queue::Position, Queue::TrackHash])
            .values_panic(vec![(i as i64).into(), id.0.to_vec().into()])
            .to_owned();

        let (sql, values) = query.build_rusqlite(sea_query::SqliteQueryBuilder);
        let params = values.as_params();

        tx.execute(&sql, params.as_slice())?;
    }

    tx.commit()?;

    Ok(())
}
