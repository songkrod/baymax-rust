// ✅ แก้ไขเพื่อให้สามารถแทนที่ `record_and_trim()` ของ `vad.rs` เดิมได้เลย
// ✅ ไม่ต้องใช้ ffmpeg, ไม่ต้อง trim ซ้ำภายหลัง
// ✅ ใช้ VAD แบบ async และหยุดอัตโนมัติเมื่อเงียบ

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use hound::{WavSpec, WavWriter, SampleFormat as HoundFormat};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};
use log::{info, warn};
use num_traits::cast::ToPrimitive;
use webrtc_vad::{Vad, VadMode, SampleRate};

/// บันทึกเสียงด้วย VAD: เริ่มเมื่อมีเสียง หยุดเมื่อเงียบ
pub fn record_and_trim(
    input_path: &str,
    max_duration_ms: u64,
    silence_timeout_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or("no input device available")?;
    let config = device.default_input_config()?;
    let stream_config: cpal::StreamConfig = config.clone().into();
    let sample_rate = stream_config.sample_rate.0;
    let channels = stream_config.channels;

    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: HoundFormat::Int,
    };

    let writer = Arc::new(Mutex::new(Some(WavWriter::create(input_path, spec)?)));
    let (tx, rx) = mpsc::channel::<Vec<i16>>();
    let (done_tx, done_rx) = mpsc::channel::<()>();

    // 🧠 VAD thread แยกออกไป
    thread::spawn(move || {
        let mut vad = Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Aggressive);
        let mut last_voice = Instant::now();
        let mut heard_voice = false;
        let start = Instant::now();

        for frame in rx {
            if let Ok(true) = vad.is_voice_segment(&frame) {
                if !heard_voice {
                    info!("👂 ได้ยินเสียงครั้งแรก! เริ่มจับเวลา silence_timeout_ms");
                    heard_voice = true;
                }
                last_voice = Instant::now();
            }

            if heard_voice && last_voice.elapsed() > Duration::from_millis(silence_timeout_ms) {
                info!("📭 เงียบเกิน {}ms → หยุด", silence_timeout_ms);
                let _ = done_tx.send(());
                break;
            }

            if start.elapsed() > Duration::from_millis(max_duration_ms) {
                info!("🛑 ยังไม่ได้ยินเสียงใน {}ms → หยุด", max_duration_ms);
                let _ = done_tx.send(());
                break;
            }
        }
    });

    // 🔈 stream อ่านเสียง เขียนลงไฟล์ ส่ง frame ไปให้ VAD
    let writer_clone = Arc::clone(&writer);
    let mut buffer = Vec::<i16>::new();
    let frame_size = 160; // 10ms @ 16kHz

    let stream = match config.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, &stream_config, writer_clone, tx, &mut buffer, frame_size)?,
        SampleFormat::I16 => build::<i16>(&device, &stream_config, writer_clone, tx, &mut buffer, frame_size)?,
        SampleFormat::U16 => build::<u16>(&device, &stream_config, writer_clone, tx, &mut buffer, frame_size)?,
        _ => return Err("Unsupported sample format".into()),
    };

    stream.play()?;
    info!("🎤 เริ่มบันทึก raw audio...");

    let deadline = Instant::now() + Duration::from_millis(max_duration_ms);
    while Instant::now() < deadline {
        if done_rx.recv_timeout(Duration::from_millis(100)).is_ok() {
            break;
        }
    }

    drop(stream);
    if let Some(writer) = writer.lock().unwrap().take() {
        writer.finalize()?;
    }
    info!("✅ เสร็จสิ้นการบันทึกเสียง raw");
    Ok(())
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    writer: Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
    tx: Sender<Vec<i16>>,
    buffer: &mut Vec<i16>,
    frame_size: usize,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: Sample + ToPrimitive + Send + 'static + cpal::SizedSample,
{
    let writer_clone = Arc::clone(&writer);
    let mut local_buf = buffer.clone();

    let stream = device.build_input_stream::<T, _, _>(
        config,
        move |data: &[T], _| {
            if let Ok(mut guard) = writer_clone.lock() {
                if let Some(ref mut writer) = *guard {
                    for &sample in data {
                        let value = sample.to_f32().unwrap_or(0.0);
                        let i16_sample = (value * i16::MAX as f32) as i16;
                        writer.write_sample(i16_sample).ok();
                        local_buf.push(i16_sample);

                        if local_buf.len() >= frame_size {
                            let frame = local_buf[..frame_size].to_vec();
                            tx.send(frame).ok();
                            local_buf.drain(..frame_size);
                        }
                    }
                }
            }
        },
        move |err| {
            warn!("⚠️ stream error: {:?}", err);
        },
        None,
    )?;
    Ok(stream)
}
