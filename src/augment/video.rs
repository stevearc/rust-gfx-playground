use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use std::path::Path;
use std::sync::mpsc::*;
use std::thread;
use std::time::*;

pub fn load_video(filename: &Path, tx: Sender<Video>) -> Result<(), ffmpeg::Error> {
    loop {
        load_video_once(filename, &tx)?;
    }
}

fn load_video_once(filename: &Path, tx: &Sender<Video>) -> Result<(), ffmpeg::Error> {
    let mut ictx = input(&filename)?;
    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let video_stream_index = input.index();

    let mut decoder = input.codec().decoder().video()?;

    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let mut receive_and_process_decoded_frames =
        |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&decoded, &mut rgb_frame)?;
                tx.send(rgb_frame)
                    .ok()
                    .ok_or(ffmpeg::Error::BufferTooSmall)?;
                // TODO: get frame schedule from ffmpeg
                thread::sleep(Duration::from_millis(16));
            }
            Ok(())
        };

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            receive_and_process_decoded_frames(&mut decoder)?;
        }
    }
    decoder.send_eof()?;
    receive_and_process_decoded_frames(&mut decoder)?;
    Ok(())
}
