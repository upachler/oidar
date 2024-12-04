# OIDAR(adio)

basis for MP3 internet radio player


## Based on

* [cpal](https://github.com/RustAudio/cpal) for audio playback
* [Symphonia](https://github.com/pdeljanov/Symphonia/blob/master/GETTING_STARTED.md) for MP3 decoding
* [hyper](https://github.com/hyperium/hyper) for reading HTTP media streams from radio stations
* [radiobrowser.info](https://www.radio-browser.info/) (REST API) for finding radio stations

## Architecture

Oidar tries to use principles from the article [Master hexagonal architecture in Rust](https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust) where applicable.

The design uses a
* Frontend: may be a GUI, a command line interface, software reading from hardware buttons, etc.
* Backend: The actual player

Frontend and backend communicate asynchronously via [channels](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html).
* frontend command channel: The frontend sends commands to this channel, like a station search request, a station selection command, or a stop command.
* backend event channel: The backend sends status changes to the frontend (and possibly other receivers), indicating changes in playback status (actually started playing, after a station was selected), change in playback status, etc.

### Backend

Uses separate threads (might be tasks, if we'd be going async):

* control thread: 
  * reads commands from frontend channel and constrols playback and the rest.
  * sends events to event channel.
* stream loader thread:
  * If current stream URL set,
    * reads chunks of stream URL and posts them into event channel
  * Awaits to commands from control thread (set URL, stop loading, shutdown)
* decoder thread:
  * decodes MP3 stream chunks received from stream loader thread into PCM data and posts them into the playback channel
* playback thread:
  * takes PCM data chunks and plays them