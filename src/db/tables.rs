use sea_query::Iden;

//
// Main Tables
//

#[derive(Iden)]
pub enum Tracks {
    Table,
    Id,
    TrackHash,
    Name,
    AlbumId,
    Duration,
    ImageHash,
}

#[derive(Iden)]
pub enum TrackSources {
    Table,
    Id,
    TrackId,
    Path,
    Size,
    Modified,
}

#[derive(Iden)]
pub enum Albums {
    Table,
    Id,
    Name,
    ImageHash,
}

#[derive(Iden)]
pub enum Artists {
    Table,
    Id,
    Name,
    ImageHash,
}

#[derive(Iden)]
pub enum Playlists {
    Table,
    Id,
    Name,
    ImageHash,
    Source,
}

//
// Join Tables
//

#[derive(Iden)]
pub enum TrackArtists {
    Table,
    TrackId,
    ArtistId,
}

#[derive(Iden)]
pub enum AlbumArtists {
    Table,
    AlbumId,
    ArtistId,
}

#[derive(Iden)]
pub enum PlaylistTracks {
    Table,
    PlaylistId,
    TrackId,
    Position,
}
