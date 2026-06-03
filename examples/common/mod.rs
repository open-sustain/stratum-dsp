use std::error::Error;
use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::audio::{AudioCodecParameters, AudioDecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, Track, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

const FALLBACK_SAMPLE_RATE: u32 = 44_100;

pub fn decode_audio_file(path: &str) -> Result<(Vec<f32>, u32), Box<dyn Error>> {
    let src = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let mut format = symphonia::default::get_probe().probe(
        &hint,
        mss,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;

    let track = format
        .default_track(TrackType::Audio)
        .ok_or("No supported audio tracks found")?;
    let track_id = track.id;
    let codec_params = audio_codec_params(track)?;
    let sample_rate = codec_params.sample_rate.unwrap_or(FALLBACK_SAMPLE_RATE);

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(codec_params, &AudioDecoderOptions::default())?;

    let mut all_samples = Vec::new();
    let mut packet_samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(SymphoniaError::ResetRequired) => {
                return Err("Decoder reset required while reading media".into());
            }
            Err(e) => return Err(Box::new(e)),
        };

        if packet.track_id != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                packet_samples.resize(decoded.samples_interleaved(), 0.0);
                decoded.copy_to_slice_interleaved(&mut packet_samples);
                append_mono_samples(&mut all_samples, &packet_samples, decoded.num_planes());
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(Box::new(e)),
        }
    }

    Ok((all_samples, sample_rate))
}

fn audio_codec_params(track: &Track) -> Result<&AudioCodecParameters, Box<dyn Error>> {
    track
        .codec_params
        .as_ref()
        .and_then(|params| params.audio())
        .ok_or_else(|| "No supported audio codec parameters found".into())
}

fn append_mono_samples(out: &mut Vec<f32>, interleaved: &[f32], channels: usize) {
    if channels == 0 {
        return;
    }

    if channels == 1 {
        out.extend_from_slice(interleaved);
        return;
    }

    out.extend(
        interleaved
            .chunks_exact(channels)
            .map(|frame| frame.iter().copied().sum::<f32>() / channels as f32),
    );
}
