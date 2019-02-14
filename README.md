# oggify
Download Spotify tracks to Ogg Vorbis (with a premium account).

This library uses [librespot](https://github.com/librespot-org/librespot). It is my first program in Rust so you may see some horrors in the way I handle tokio, futures and such.

# Usage
```
oggify user password < tracks_list
```
The program takes 2 arguments, your Spotify Premium user and password, then reads from stdin and looks for a track URL or URI in each line. The two formats are those you get with the track menu items "Share->Copy Song Link" or "Share->Copy Song URI" in the Spotify client, for example `open.spotify.com/track/1xPQDRSXDN5QJWm7qHg5Ku` or `spotify:track:1xPQDRSXDN5QJWm7qHg5Ku`. For example,
