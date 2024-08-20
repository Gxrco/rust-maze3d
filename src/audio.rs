use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

pub struct AudioPlayer {
    sink: Arc<Mutex<Sink>>,
    _stream: OutputStream,
}

impl AudioPlayer {
    pub fn new(music_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let file = File::open(music_file)?;
        let source = Decoder::new(BufReader::new(file))?;
        sink.append(source);
        sink.set_volume(0.5);
        sink.pause();

        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
            _stream: stream,
        })
    }

    pub fn play(&self) {
        if let Ok(sink) = self.sink.lock() {
            sink.play();
        } else {
            eprintln!("Failed to lock the sink for playback.");
        }
    }

    pub fn pause(&self) {
        if let Ok(sink) = self.sink.lock() {
            sink.pause();
        } else {
            eprintln!("Failed to lock the sink to pause playback.");
        }
    }
}
