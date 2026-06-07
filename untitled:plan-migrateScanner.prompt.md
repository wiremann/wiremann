# WIREMANN SQLITE MVP PLAN

---

# GOAL

Achieve:

```text
scanner -> sqlite -> fetch -> ui print
```

using:

* normalized relational schema
* proper transactions
* pooled sqlite connections
* existing GPUI async architecture

WITHOUT:

* redesigning app architecture
* adding repositories/ORMs
* adding db threads/channels
* overengineering abstractions

---

# HIGH LEVEL ARCHITECTURE

Keep existing flow EXACTLY:

```text
Scanner Thread
    ↓
ScannerEvent::UpsertTracks
    ↓
Controller Handler
    ↓
cx.spawn + smol::unblock
    ↓
SQLite query layer
    ↓
DB write
```

This architecture is already good.

Do not redesign it.

---

# PHASE 1

# SCHEMA FIXES

Before implementing queries.

---

## 1. ADD FOREIGN KEYS

In `m0001_init.rs`.

### tracks.album_id

```rust
.foreign_key(
    ForeignKey::create()
        .from(Tracks::Table, Tracks::AlbumId)
        .to(Albums::Table, Albums::Id)
        .on_delete(ForeignKeyAction::SetNull)
)
```

---

### track_sources.track_id

```rust
.on_delete(ForeignKeyAction::Cascade)
```

---

### track_artists

Cascade delete both sides.

---

### album_artists

Cascade delete both sides.

---

### playlist_tracks

Cascade delete both sides.

---

# 2. ADD UNIQUE CONSTRAINTS

---

## albums.name

```rust
.col(
    ColumnDef::new(Albums::Name)
        .text()
        .not_null()
        .unique_key()
)
```

---

## artists.name

Same.

---

# 3. KEEP PLAYLIST UUIDS

Do NOT change playlists to integers.

Current:

```rust
TEXT PRIMARY KEY
```

is perfectly fine.

---

# 4. KEEP TRACK HASH

BUT:

* not relational identity
* not FK target
* not primary dedupe mechanism

Use ONLY for:

* lyrics cache
* image cache
* filesystem assets

---

# PHASE 2

# CREATE QUERY LAYER

Create:

```text
src/db/queries/
├── mod.rs
└── scanner.rs
```

Nothing else yet.

No models folder yet.

No repositories.

No traits.

No generic DB layer.

---

# queries/mod.rs

```rust
pub mod scanner;
```

Done.

---

# PHASE 3

# IMPLEMENT INSERTION PIPELINE

Inside:

```text
src/db/queries/scanner.rs
```

---

# PUBLIC FUNCTIONS

ONLY THESE:

---

## 1.

```rust
pub fn upsert_scanned_tracks(
    conn: &mut Connection,
    tracks: &[ScannedTrack],
) -> anyhow::Result<()>
```

Batch insertion.

Uses ONE transaction.

This is the main scanner entry point.

---

## 2.

```rust
pub fn get_all_tracks(
    conn: &Connection,
) -> anyhow::Result<Vec<DbTrack>>
```

Used for UI testing.

---

# INTERNAL HELPERS

Keep private for now.

---

## upsert_album

```rust
fn upsert_album(
    tx: &Transaction,
    name: &str,
) -> anyhow::Result<Option<i64>>
```

Behavior:

* insert if missing
* fetch existing id otherwise

Returns SQLite integer id.

---

## upsert_artist

```rust
fn upsert_artist(
    tx: &Transaction,
    name: &str,
) -> anyhow::Result<i64>
```

---

## upsert_track

```rust
fn upsert_track(
    tx: &Transaction,
    track: &ScannedTrack,
    album_id: Option<i64>,
) -> anyhow::Result<i64>
```

Returns integer track id.

---

## upsert_track_source

```rust
fn upsert_track_source(
    tx: &Transaction,
    track_id: i64,
    source: &ScannedTrackSource,
) -> anyhow::Result<()>
```

Uses:

```sql
ON CONFLICT(path)
DO UPDATE
```

---

## insert_track_artist

```rust
fn insert_track_artist(
    tx: &Transaction,
    track_id: i64,
    artist_id: i64,
) -> anyhow::Result<()>
```

Uses:

```sql
INSERT OR IGNORE
```

---

# PHASE 4

# INSERT FLOW

Inside:

```rust
upsert_scanned_tracks()
```

---

## START TRANSACTION

```rust
let tx = conn.transaction()?;
```

---

## FOR EACH TRACK

---

### 1. upsert album

Get `album_id`.

---

### 2. upsert artists

Collect `artist_ids`.

---

### 3. upsert track

Get `track_id`.

---

### 4. upsert source

Insert/update path metadata.

---

### 5. insert track_artists

Create relations.

---

## COMMIT

```rust
tx.commit()?;
```

DONE.

---

# PHASE 5

# DB TRACK STRUCT

Inside `scanner.rs` for now.

```rust
pub struct DbTrack {
    pub id: i64,
    pub title: String,
    pub album: Option<String>,
    pub artists: Vec<String>,
    pub duration: Duration,
}
```

No separate models folder yet.

Too early.

---

# PHASE 6

# IMPLEMENT FETCH QUERY

Implement:

```rust
get_all_tracks()
```

Query:

* tracks
* albums
* track_artists
* artists

Group artists in Rust.

Return `Vec<DbTrack>`.

This query exists ONLY for testing right now.

---

# PHASE 7

# WIRE INTO SCANNER HANDLER

Current handler:

```rust
for (track, job_id) in tracks
```

CHANGE EVENT PAYLOAD FIRST.

Current payload is cursed because:

* `u64` scan job id
* not meaningful for DB

For MVP:

CHANGE:

```rust
ScannerEvent::UpsertTracks(Vec<(ScannedTrack, u64)>)
```

TO:

```rust
ScannerEvent::UpsertTracks(Vec<ScannedTrack>)
```

Temporarily remove playlist logic from scanner insertion.

Playlist integration comes AFTER DB bringup.

This massively simplifies MVP.

---

# HANDLER FLOW

Inside handler:

```rust
let db = cx.global::<Database>().clone();

cx.spawn(async move |_cx| {
    smol::unblock(move || {
        let mut conn = db.pool().get()?;

        crate::db::queries::scanner::upsert_scanned_tracks(
            &mut conn,
            &tracks,
        )?;

        anyhow::Ok(())
    })
    .await
    .unwrap();
})
.detach();
```

---

# PHASE 8

# UI TEST

Anywhere convenient:

```rust
let db = cx.global::<Database>().clone();

cx.spawn(async move |_cx| {
    let tracks = smol::unblock(move || {
        let conn = db.pool().get()?;

        crate::db::queries::scanner::get_all_tracks(&conn)
    })
    .await
    .unwrap();

    println!("{tracks:#?}");
})
.detach();
```

DONE.

Now ye have:

* scanner persistence
* normalized DB
* reads working
* async architecture preserved
* transaction batching
* proper sqlite integration

That’s the REAL MVP ⚓

---

# THINGS INTENTIONALLY POSTPONED

NOT NOW:

❌ playlist DB integration
❌ image queries
❌ thumbnails
❌ artwork pipeline
❌ full library hydration
❌ replacing AppState fully
❌ repositories
❌ generic abstractions
❌ reactive DB systems
❌ caching layers

Those happen AFTER the DB pipeline is proven working.

---

# IMPLEMENTATION ORDER

Follow THIS exact order:

---

## 1.

Fix schema:

* FK constraints
* UNIQUE constraints

Run:

```bash
cargo check
```

---

## 2.

Create:

```text
queries/mod.rs
queries/scanner.rs
```

Run:

```bash
cargo check
```

---

## 3.

Implement:

* upsert_album
* upsert_artist

Run:

```bash
cargo check
```

---

## 4.

Implement:

* upsert_track
* upsert_track_source
* insert_track_artist

Run:

```bash
cargo check
```

---

## 5.

Implement:

```rust
upsert_scanned_tracks()
```

Run:

```bash
cargo check
```

---

## 6.

Hook scanner handler to DB.

Run:

```bash
cargo check
```

---

## 7.

Implement:

```rust
get_all_tracks()
```

---

## 8.

Print from UI.

---
