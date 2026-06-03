//! Example: Analyze multiple audio files in parallel
//!
//! Usage:
//!   cargo run --release --example analyze_batch -- [--jobs N] [--json] <file1> <file2> ...
//!
//! Notes:
//! - Parallelism is across files (batch-level). Each file analysis is still single-threaded.
//! - Default workers: (available CPU threads - 1), keeping one core free for the system.

use rayon::prelude::*;
use std::env;
use std::time::Instant;
use stratum_dsp::{analyze_audio, compute_confidence, AnalysisConfig};

mod common;

use common::decode_audio_file;

fn default_jobs() -> usize {
    let n = std::thread::available_parallelism()
        .map(|v| v.get())
        .unwrap_or(1);
    std::cmp::max(1, n.saturating_sub(1))
}

fn percentile(mut xs: Vec<f32>, p: f32) -> Option<f32> {
    if xs.is_empty() {
        return None;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((xs.len() - 1) as f32 * p.clamp(0.0, 1.0)).round() as usize;
    Some(xs[idx.min(xs.len() - 1)])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    let mut json = false;
    let mut jobs: Option<usize> = None;
    let mut paths: Vec<String> = Vec::new();

    while let Some(a) = args.first().cloned() {
        args.remove(0);
        match a.as_str() {
            "--json" => json = true,
            "--jobs" => {
                let v = args
                    .first()
                    .ok_or("--jobs requires a value")?
                    .parse::<usize>()?;
                args.remove(0);
                jobs = Some(std::cmp::max(1, v));
            }
            "--help" | "-h" => {
                eprintln!(
                    "Usage: analyze_batch [--jobs N] [--json] <file1> <file2> ...\n\
                     \n\
                     --jobs N   Parallel workers (default: CPU-1)\n\
                     --json     Emit one JSON object per line (JSONL)\n"
                );
                return Ok(());
            }
            _ => paths.push(a),
        }
    }

    if paths.is_empty() {
        eprintln!("ERROR: Provide at least one audio file path. Use --help for usage.");
        std::process::exit(2);
    }

    let jobs = jobs.unwrap_or_else(default_jobs);
    eprintln!("Batch: {} files, jobs={}", paths.len(), jobs);

    let config = AnalysisConfig::default();

    let t0 = Instant::now();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .expect("Failed to build rayon thread pool");

    #[derive(Clone)]
    struct ItemOut {
        path: String,
        ok: bool,
        bpm: f32,
        bpm_conf: f32,
        key: String,
        key_conf: f32,
        processing_ms: f32,
        tempogram_multi_res_triggered: Option<bool>,
        tempogram_multi_res_used: Option<bool>,
        tempogram_percussive_triggered: Option<bool>,
        tempogram_percussive_used: Option<bool>,
        error: Option<String>,
    }

    let outs: Vec<ItemOut> = pool.install(|| {
        paths
            .par_iter()
            .map(|path| {
                let path_s = path.clone();
                let decoded = decode_audio_file(&path_s);
                match decoded {
                    Ok((samples, sr)) => {
                        let r = analyze_audio(&samples, sr, config.clone());
                        match r {
                            Ok(res) => {
                                let conf = compute_confidence(&res);
                                ItemOut {
                                    path: path_s,
                                    ok: true,
                                    bpm: res.bpm,
                                    bpm_conf: conf.bpm_confidence,
                                    key: res.key.name().to_string(),
                                    key_conf: conf.key_confidence,
                                    processing_ms: res.metadata.processing_time_ms,
                                    tempogram_multi_res_triggered: res
                                        .metadata
                                        .tempogram_multi_res_triggered,
                                    tempogram_multi_res_used: res.metadata.tempogram_multi_res_used,
                                    tempogram_percussive_triggered: res
                                        .metadata
                                        .tempogram_percussive_triggered,
                                    tempogram_percussive_used: res
                                        .metadata
                                        .tempogram_percussive_used,
                                    error: None,
                                }
                            }
                            Err(e) => ItemOut {
                                path: path_s,
                                ok: false,
                                bpm: 0.0,
                                bpm_conf: 0.0,
                                key: "".to_string(),
                                key_conf: 0.0,
                                processing_ms: 0.0,
                                tempogram_multi_res_triggered: None,
                                tempogram_multi_res_used: None,
                                tempogram_percussive_triggered: None,
                                tempogram_percussive_used: None,
                                error: Some(format!("analysis failed: {e}")),
                            },
                        }
                    }
                    Err(e) => ItemOut {
                        path: path_s,
                        ok: false,
                        bpm: 0.0,
                        bpm_conf: 0.0,
                        key: "".to_string(),
                        key_conf: 0.0,
                        processing_ms: 0.0,
                        tempogram_multi_res_triggered: None,
                        tempogram_multi_res_used: None,
                        tempogram_percussive_triggered: None,
                        tempogram_percussive_used: None,
                        error: Some(format!("decode failed: {e}")),
                    },
                }
            })
            .collect()
    });

    if json {
        for o in &outs {
            if o.ok {
                println!(
                    "{{\"file\":{},\"bpm\":{:.2},\"bpm_confidence\":{:.4},\"key\":{},\"key_confidence\":{:.4},\"processing_time_ms\":{:.2},\"tempogram_multi_res_triggered\":{},\"tempogram_multi_res_used\":{},\"tempogram_percussive_triggered\":{},\"tempogram_percussive_used\":{}}}",
                    serde_json::to_string(&o.path).unwrap(),
                    o.bpm,
                    o.bpm_conf,
                    serde_json::to_string(&o.key).unwrap(),
                    o.key_conf,
                    o.processing_ms,
                    o.tempogram_multi_res_triggered.map(|v| v.to_string()).unwrap_or("null".to_string()),
                    o.tempogram_multi_res_used.map(|v| v.to_string()).unwrap_or("null".to_string()),
                    o.tempogram_percussive_triggered.map(|v| v.to_string()).unwrap_or("null".to_string()),
                    o.tempogram_percussive_used.map(|v| v.to_string()).unwrap_or("null".to_string()),
                );
            } else {
                println!(
                    "{{\"file\":{},\"error\":{}}}",
                    serde_json::to_string(&o.path).unwrap(),
                    serde_json::to_string(o.error.as_deref().unwrap_or("unknown error")).unwrap()
                );
            }
        }
    } else {
        for (idx, o) in outs.iter().enumerate() {
            if o.ok {
                println!(
                    "[{}/{}] {}: BPM={:.2} (conf={:.3}) Key={} (conf={:.3}) time={:.2}ms",
                    idx + 1,
                    outs.len(),
                    o.path,
                    o.bpm,
                    o.bpm_conf,
                    o.key,
                    o.key_conf,
                    o.processing_ms
                );
            } else {
                println!(
                    "[{}/{}] {}: ERROR: {}",
                    idx + 1,
                    outs.len(),
                    o.path,
                    o.error.as_deref().unwrap_or("unknown error")
                );
            }
        }
    }

    let ok_times: Vec<f32> = outs
        .iter()
        .filter(|o| o.ok)
        .map(|o| o.processing_ms)
        .collect();
    let wall = t0.elapsed();
    let wall_ms = wall.as_secs_f64() * 1000.0;

    eprintln!(
        "Done: ok={}/{} wall={:.0}ms",
        ok_times.len(),
        outs.len(),
        wall_ms
    );
    if !ok_times.is_empty() {
        let mean = ok_times.iter().sum::<f32>() / ok_times.len() as f32;
        let p50 = percentile(ok_times.clone(), 0.50).unwrap_or(mean);
        let p90 = percentile(ok_times.clone(), 0.90).unwrap_or(mean);
        let min = ok_times.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = ok_times.iter().cloned().fold(0.0, f32::max);
        eprintln!(
            "processing_time_ms: mean={:.2} p50={:.2} p90={:.2} min={:.2} max={:.2}",
            mean, p50, p90, min, max
        );
    }

    Ok(())
}
