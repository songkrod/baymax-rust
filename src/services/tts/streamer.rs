// src/services/tts/streamer.rs
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::sync::Arc;
use tokio::task;

/// เล่นเสียงแต่ละ chunk แบบ async และไม่ block thread หลัก
pub async fn play_chunks_in_parallel(audio_datas: Vec<Vec<u8>>) {
    for audio in audio_datas {
        let data = audio.clone();
        task::spawn_blocking(move || {
            let cursor = Cursor::new(data);
            if let Ok((_stream, handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&handle) {
                    if let Ok(source) = Decoder::new(cursor) {
                        sink.append(source);
                        sink.sleep_until_end();
                    }
                }
            }
        });
    }
}