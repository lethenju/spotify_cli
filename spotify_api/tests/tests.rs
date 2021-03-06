extern crate spotify_api;
use spotify_api::EasyAPI;
#[cfg(test)]
#[test]
fn initialize_handle() {
    let mut handle = EasyAPI::new();
    handle.refresh();
}

#[test]
fn search_tracks() {
    let mut handle = EasyAPI::new();
    handle.refresh();
    let billie_jean_results = handle.search_track("billie jean").unwrap();

    assert!(billie_jean_results.len() > 0);
    for track in billie_jean_results {
        println!("TRACKS {} : {}", track.id, track.name);
    }
}

#[test]
fn search_albums() {
    let mut handle = EasyAPI::new();
    handle.refresh();
    let thriller_results = handle.search_album("thriller").unwrap();
    assert!(thriller_results.len() > 0);
    for album in thriller_results {
        println!("ALBUMS {} : {}", album.id, album.name);
    }
}

#[test]
fn search_artists() {
    let mut handle = EasyAPI::new();
    handle.refresh();
    let mj_results = handle.search_artist("Michael Jackson").unwrap();
    assert!(mj_results.len() > 0);
    for artist in mj_results {
        println!("ARTIST {} : {}", artist.id, artist.name);
    }
}

#[test]
fn search_playlists() {
    let mut handle = EasyAPI::new();
    handle.refresh();
    let mj_results = handle.search_playlist("Michael Jackson").unwrap();
    assert!(mj_results.len() > 0);
    for playlist in mj_results {
        println!("PLAYLIST {} : {}", playlist.id, playlist.name);
    }
}

#[test]
fn play_without_context() {
    let mut handle = EasyAPI::new();
    handle.refresh();
    let billie_jean_results = handle.search_track("billie jean").unwrap();
    handle.play_track(&billie_jean_results[0], None).unwrap();
}

#[test]
fn play_with_context() {
    let mut handle = EasyAPI::new();
    handle.refresh();
    let thriller_results = handle.search_album("thriller").unwrap();
    ;
    let track_results = handle
        .get_tracks_from_album(&thriller_results[0].id)
        .unwrap();
    handle
        .play_track(&track_results[0], Some(&thriller_results[0]))
        .unwrap();
}

#[test]
fn get_tracks_from_album() {
    let mut handle = EasyAPI::new();
    handle.refresh();
    let thriller_results = handle.search_album("thriller").unwrap();
    let track_results = handle
        .get_tracks_from_album(&thriller_results[0].id)
        .unwrap();;

    assert!(track_results.len() > 0);
    assert!(track_results[3].name == "Thriller"); // the 4rth track of the Thriller album is Thriller
}
