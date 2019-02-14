extern crate tokio_core;
extern crate regex;
extern crate librespot_core;
extern crate librespot_metadata;
extern crate librespot_audio;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate scoped_threadpool;

use std::time::Duration;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{self, Read, BufRead, Result};
use tokio_core::reactor::Core;
use regex::Regex;
use env_logger::{Builder, Env};
use scoped_threadpool::Pool;

use librespot_core::authentication::Credentials;
use librespot_core::config::SessionConfig;
use librespot_core::session::Session;
use librespot_core::spotify_id::SpotifyId;
use librespot_metadata::{Metadata, FileFormat, Track, Artist};
use librespot_audio::{AudioDecrypt, AudioFile};

fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Vec<_> = env::args().collect();
    assert!(args.len() == 3, "Usage: {} USERNAME PASSWORD < tracks_file", args[0]);

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let session_config = SessionConfig::default();
    let credentials = Credentials::with_password(args[1].to_owned(), args[2].to_owned());
    info!("Connecting ...");
    let session = core
        .run(Session::connect(session_config, credentials, None, handle))
        .unwrap();
    info!("Connected!");

    let mut threadpool = Pool::new(1);

    let spotify_uri = Regex::new(r"spotify:track:([[:alnum:]]+)").unwrap();
    let spotify_url = Regex::new(r"open\.spotify\.com/track/([[:alnum:]]+)").unwrap();

    io::stdin().lock().lines()
        .filter_map(|line|
            line.ok().and_then(|str|
                spotify_uri.captures(&str).or(spotify_url.captures(&str))
                    .or_else(|| { warn!("Cannot parse track from string {}", str); None })
                    .and_then(|capture|SpotifyId::from_base62(&capture[1]).ok())))
        .for_each(|id|{
            info!("Getting track {}...", id.to_base62());
            let track = core.run(Track::get(&session, id)).expect("Cannot get track metadata");
            let artists_strs: Vec<_> = track.artists.iter().map(|id|core.run(Artist::get(&session, *id)).expect("Cannot get artist metadata").name).collect();
            let artists_display = artists_strs.join(", ");
            let fname = format!("{} - {}.ogg", artists_display, track.name);
            debug!("File formats: {}", track.files.keys().map(|filetype|format!("{:?}", filetype)).collect::<Vec<_>>().join(" "));
            let file_id = track.files.get(&FileFormat::OGG_VORBIS_320)
                .or(track.files.get(&FileFormat::OGG_VORBIS_160))
                .or(track.files.get(&FileFormat::OGG_VORBIS_96))
                .expect("Could not find a OGG_VORBIS format for the track.");
            let key = core.run(session.audio_key().request(track.id, *file_id)).expect("Cannot get audio key");
            let mut encrypted_file = core.run(AudioFile::open(&session, *file_id)).unwrap();
            let mut buffer = Vec::new();
            let mut read_all: Result<usize> = Ok(0);
            let fetched = AtomicBool::new(false);
            threadpool.scoped(|scope|{
                scope.execute(||{
                    read_all = encrypted_file.read_to_end(&mut buffer);
                    fetched.store(true, Ordering::Release);
                });
                while !fetched.load(Ordering::Acquire) {
                    core.turn(Some(Duration::from_millis(100)));
                }
            });
            read_all.expect("Cannot read file stream");
            let mut decrypted_buffer = Vec::new();
            AudioDecrypt::new(key, &buffer[..]).read_to_end(&mut decrypted_buffer).expect("Cannot decrypt stream");
            std::fs::write(&fname, &decrypted_buffer[0xa7..]).expect("Cannot write decrypted track");
            info!("Filename: {}", fname);
        });
}
