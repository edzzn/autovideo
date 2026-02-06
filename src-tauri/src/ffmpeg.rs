use crate::models::PipelineStage;
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

pub struct FFmpegProcess {
    process: Child,
    current_stage: Option<PipelineStage>,
}

impl FFmpegProcess {
    pub fn new(command: &str, args: &[&str]) -> Result<Self, String> {
        let mut cmd = Command::new(command);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        let process = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn FFmpeg: {}", e))?;

        let mut ffmpeg = FFmpegProcess {
            process,
            current_stage: None,
        };

        ffmpeg.set_current_stage(PipelineStage::DetectSilences);

        Ok(ffmpeg)
    }

    fn set_current_stage(&mut self, stage: PipelineStage) {
        self.current_stage = Some(stage);
    }

    pub fn get_current_stage(&self) -> Option<PipelineStage> {
        self.current_stage
    }

    pub fn read_stderr(&mut self) -> Result<String, String> {
        let stderr = self.process.stderr.as_mut().ok_or("No stderr available")?;

        let mut buffer = String::new();
        let mut reader = BufReader::new(stderr);

        reader
            .read_line(&mut buffer)
            .map_err(|e| format!("Failed to read stderr: {}", e))?;

        Ok(buffer)
    }

    pub fn wait_for_completion(mut self) -> Result<(), String> {
        let status = self
            .process
            .wait()
            .map_err(|e| format!("Failed to wait for FFmpeg: {}", e))?;

        if !status.success() {
            return Err("FFmpeg command failed".to_string());
        }

        Ok(())
    }

    pub fn kill(&mut self) {
        let _ = self.process.kill();
    }
}

/// Returns Vec of (silence_start, silence_end) tuples
pub fn parse_silencedetect(output: &str) -> Vec<(f64, f64)> {
    let mut silences = Vec::new();

    let start_pattern = Regex::new(r"silence_start:\s*([\d.]+)").unwrap();
    let end_pattern = Regex::new(r"silence_end:\s*([\d.]+)").unwrap();

    let mut current_start: Option<f64> = None;

    for line in output.lines() {
        if let Some(captures) = start_pattern.captures(line) {
            if let Ok(start) = captures[1].parse::<f64>() {
                current_start = Some(start);
            }
        }
        if let Some(captures) = end_pattern.captures(line) {
            if let Ok(end) = captures[1].parse::<f64>() {
                if let Some(start) = current_start.take() {
                    silences.push((start, end));
                }
            }
        }
    }

    silences
}

pub fn extract_audio(
    input_path: &str,
    output_path: &str,
    sample_rate: u32,
    num_channels: u16,
) -> Result<(), String> {
    let sample_rate_str = sample_rate.to_string();
    let num_channels_str = num_channels.to_string();
    let args = vec![
        "-i",
        input_path,
        "-ar",
        &sample_rate_str,
        "-ac",
        &num_channels_str,
        "-f",
        "f32le",
        "-acodec",
        "pcm_f32le",
        "-y",
        output_path,
    ];

    let mut process = Command::new("ffmpeg")
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to spawn FFmpeg: {}", e))?;

    let status = process
        .wait()
        .map_err(|e| format!("Failed to wait for FFmpeg: {}", e))?;

    if !status.success() {
        return Err("FFmpeg extraction failed".to_string());
    }

    Ok(())
}

pub fn get_video_duration(input_path: &str) -> Result<f64, String> {
    let args = vec!["-i", input_path, "-t", "0.000001", "-f", "null", "-"];

    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to get video duration: {}", e))?;

    // FFmpeg outputs to stderr, not stdout
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Match duration with flexible millisecond precision
    let pattern = Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.(\d+)").unwrap();

    for line in stderr.lines() {
        if let Some(captures) = pattern.captures(line) {
            if let (Ok(h), Ok(m), Ok(s), Ok(ms)) = (
                captures[1].parse::<f64>(),
                captures[2].parse::<f64>(),
                captures[3].parse::<f64>(),
                captures[4].parse::<f64>(),
            ) {
                // ms is in centiseconds (2 digits) or more precision
                let ms_divisor = 10_f64.powi(captures[4].len() as i32);
                let duration = h * 3600.0 + m * 60.0 + s + ms / ms_divisor;
                eprintln!("📹 Video duration: {:.2}s ({:02}:{:02}:{:02})", duration, h as i32, m as i32, s as i32);
                return Ok(duration);
            }
        }
    }

    eprintln!("❌ Failed to parse duration from FFmpeg output");
    eprintln!("FFmpeg stderr: {}", stderr);
    Err("Could not parse duration".to_string())
}

pub fn enhance_audio(input_path: &str, output_path: &str) -> Result<(), String> {
    let args = vec![
        "-i",
        input_path,
        "-af",
        "afftdn=nf=-25,loudnorm=I=-16:TP=-1.5:LRA=11",
        "-c:v",
        "copy",
        "-y",
        output_path,
    ];

    run_ffmpeg_command(args)
}

/// Returns Vec of (silence_start, silence_end) tuples
pub fn detect_silences(
    input_path: &str,
    threshold_db: f64,
    min_duration: f64,
) -> Result<Vec<(f64, f64)>, String> {
    // threshold_db is already negative (e.g., -30.0), so don't add another minus sign
    let silence_filter = format!("silencedetect=noise={}dB:d={}", threshold_db, min_duration);
    let args = vec!["-i", input_path, "-af", &silence_filter, "-f", "null", "-"];

    eprintln!("🔍 Detecting silences with filter: {}", silence_filter);
    let output = run_ffmpeg_command_raw(args)?;

    let silences = parse_silencedetect(&output);
    let total_silence: f64 = silences.iter().map(|(s, e)| e - s).sum();
    eprintln!("📊 Found {} silence segments totaling {:.2}s", silences.len(), total_silence);
    Ok(silences)
}

pub fn cut_silences_and_export(
    input_path: &str,
    keep_ranges: Vec<(f64, f64)>,
    output_path: &str,
    enhance_audio: bool,
) -> Result<(), String> {
    // Build the select expression: between(t,start1,end1)+between(t,start2,end2)+...
    let keep_expr: String = keep_ranges
        .iter()
        .map(|(start, end)| format!("between(t,{},{})", start, end))
        .collect::<Vec<_>>()
        .join("+");

    let select_expr = format!("select='{}',setpts=N/FRAME_RATE/TB", keep_expr);

    // Build audio filter chain - aselect + optional enhancement
    let aselect_base = format!("aselect='{}',asetpts=N/SR/TB", keep_expr);
    let audio_filter = if enhance_audio {
        format!("{},afftdn=nf=-25,loudnorm=I=-16:TP=-1.5:LRA=11", aselect_base)
    } else {
        aselect_base
    };

    eprintln!("🎬 Video filter: {}", select_expr);
    eprintln!("🔊 Audio filter: {}", audio_filter);

    let args = vec![
        "-i", input_path,
        "-vf", &select_expr,
        "-af", &audio_filter,
        "-c:v", "h264_videotoolbox",
        "-b:v", "8M",
        "-maxrate", "10M",
        "-bufsize", "16M",
        "-profile:v", "high",
        "-c:a", "aac",
        "-b:a", "192k",
        "-ar", "44100",
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        "-y", output_path,
    ];

    run_ffmpeg_command(args)
}

/// Copy video with re-encoded audio (no video processing)
pub fn copy_video(input_path: &str, output_path: &str) -> Result<(), String> {
    let args = vec![
        "-i", input_path,
        "-c:v", "copy",
        "-c:a", "aac",
        "-b:a", "192k",
        "-ar", "44100",
        "-movflags", "+faststart",
        "-y", output_path,
    ];

    run_ffmpeg_command(args)
}

fn run_ffmpeg_command(args: Vec<&str>) -> Result<(), String> {
    run_ffmpeg_command_raw(args).map(|_| ())
}

fn run_ffmpeg_command_raw(args: Vec<&str>) -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    // FFmpeg outputs most info (including silencedetect) to stderr
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!("FFmpeg failed: {}", stderr));
    }

    Ok(stderr)
}
