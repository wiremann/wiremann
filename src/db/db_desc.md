## Main Tables

### Tracks

| Column | Type | Description |
|----------|----------|----------|
| id | INTEGER PRIMARY KEY | Internal database ID |
| track_hash | BLOB UNIQUE | Stable track hash |
| name | TEXT | Track title |
| album_id | INTEGER | FK → Albums.id |
| duration | INTEGER | Duration (ms) |
| image_hash | BLOB | Artwork hash |

---

### Track Sources

| Column | Type | Description |
|----------|----------|----------|
| id | INTEGER PRIMARY KEY | Source ID |
| track_id | INTEGER | FK → Tracks.id |
| path | TEXT UNIQUE | File path |
| size | INTEGER | File size |
| modified | INTEGER | Last modified timestamp |

---

### Artists

| Column | Type | Description |
|----------|----------|----------|
| id | INTEGER PRIMARY KEY | Artist ID |
| name | TEXT | Artist name |
| image_hash | BLOB | Artwork hash |

---

### Albums

| Column | Type | Description |
|----------|----------|----------|
| id | INTEGER PRIMARY KEY | Album ID |
| name | TEXT | Album name |
| image_hash | BLOB | Artwork hash |
| duration | INTEGER | Total duration (ms) |

---

### Playlists

| Column | Type | Description |
|----------|----------|----------|
| id | INTEGER PRIMARY KEY | Playlist ID |
| name | TEXT | Playlist name |
| image_hash | BLOB | Artwork hash |
| duration | INTEGER | Total duration (ms) |
| source | TEXT | User / Imported / Generated |

---

## Join Tables

### Track Artists

| Column | Type | Description |
|----------|----------|----------|
| track_id | INTEGER | FK → Tracks.id |
| artist_id | INTEGER | FK → Artists.id |

---

### Album Artists

| Column | Type | Description |
|----------|----------|----------|
| album_id | INTEGER | FK → Albums.id |
| artist_id | INTEGER | FK → Artists.id |

---

### Playlist Tracks

| Column | Type | Description |
|----------|----------|----------|
| playlist_id | INTEGER | FK → Playlists.id |
| track_id | INTEGER | FK → Tracks.id |
| position | INTEGER | Order inside playlist |

---

## Relationships

Tracks
└── many Track Sources

Albums
└── many Tracks

Tracks
└── many-to-many Artists

Albums
└── many-to-many Artists

Playlists
└── many-to-many Tracks
