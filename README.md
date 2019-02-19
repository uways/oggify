# oggify
Download Spotify tracks to Ogg Vorbis (with a premium account).

This library uses [librespot](https://github.com/librespot-org/librespot). It is my first program in Rust so you may see some horrors in the way I handle tokio, futures and such.

# Usage
To download a number of tracks as `"artists" - "title".ogg`, run
```
oggify "spotify-premium-user" "spotify-premium-password" < tracks_list
```
Oggify reads from stdin and looks for a track URL or URI in each line. The two formats are those you get with the track menu items "Share->Copy Song Link" or "Share->Copy Song URI" in the Spotify client, for example `open.spotify.com/track/1xPQDRSXDN5QJWm7qHg5Ku` or `spotify:track:1xPQDRSXDN5QJWm7qHg5Ku`.

## Helper script
A second form of invocation of oggify is
```
oggify "spotify-premium-user" "spotify-premium-password" "helper_script" < tracks_list
```
In this form `helper_script` is invoked for each new track:
```
helper_script "spotify_id" "title" "album" "artist1" ["artist2"...] < ogg_stream
```
The script `tag_ogg` in the source tree can be used to automatically add the track information (spotify ID, title, album, artists) as vorbis comments.

### Converting to MP3
Use `oggify` with the `tag_ogg` helper script as described above, then convert with ffmpeg:
```
for ogg in *.ogg; do
	ffmpeg -i "$ogg" -map_metadata 0:s:0 -id3v2_version 3 -codec:a libmp3lame -qscale:a 2 "$(basename "$ogg" .ogg).mp3"
done
```
