use crate::models::{PipelineConfig, PipelineStage};
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

pub fn parse_silencedetect(output: &str) -> Vec<f64> {
    let mut silences = Vec::new();

    let pattern = Regex::new(r"silence_start:\s*([\d.]+)").unwrap();
    let mut timestamps = Vec::new();

    for line in output.lines() {
        if let Some(captures) = pattern.captures(line) {
            if let Ok(start) = captures[1].parse::<f64>() {
                timestamps.push(start);
            }
        }
    }

    for i in (0..timestamps.len()).step_by(2) {
        if i + 1 < timestamps.len() {
            silences.push(timestamps[i + 1] - timestamps[i]);
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

    let mut process = Command::new("binaries/ffmpeg")
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

    let output = Command::new("binaries/ffmpeg")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to get video duration: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pattern = Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap();

    for line in stdout.lines() {
        if let Some(captures) = pattern.captures(line) {
            if let (Ok(h), Ok(m), Ok(s), Ok(ms)) = (
                captures[1].parse::<f64>(),
                captures[2].parse::<f64>(),
                captures[3].parse::<f64>(),
                captures[4].parse::<f64>(),
            ) {
                return Ok(h * 3600.0 + m * 60.0 + s + ms / 100.0);
            }
        }
    }

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

pub fn detect_silences(
    input_path: &str,
    threshold_db: f64,
    min_duration: f64,
) -> Result<Vec<f64>, String> {
    let silence_filter = format!("silencedetect=noise=-{}dB:d={}", threshold_db, min_duration);
    let args = vec!["-i", input_path, "-af", &silence_filter, "-f", "null", "-"];

    let output = run_ffmpeg_command_raw(args)?;

    let silences = parse_silencedetect(&output);
    Ok(silences)
}

pub fn cut_silences_and_export(
    input_path: &str,
    keep_ranges: Vec<(f64, f64)>,
    output_path: &str,
    config: &PipelineConfig,
) -> Result<(), String> {
    let mut args = vec!["-i", input_path];

    let keep_filter: Vec<String> = keep_ranges
        .iter()
        .map(|(start, end)| format!("between(t,{},{}):{}", start, end, start))
        .collect();

    let keep_expr = keep_filter.join("+");
    let select_expr = format!("select='{}',setpts=N/FRAME_RATE/TB", keep_expr);
    let aselect_expr = format!("aselect='{}',asetpts=N/SR/TB", keep_expr);

    args.extend_from_slice(&["-vf", &select_expr]);
    args.extend_from_slice(&["-af", &aselect_expr]);

    if config.enhance_audio {
        let audio_filters = "afftdn=nf=-25,loudnorm=I=-16:TP=-1.5:LRA=11";
        args.extend_from_slice(&["-af", audio_filters]);
    }

    args.extend_from_slice(&[
        "-c:v",
        "h264_videotoolbox",
        "-b:v",
        "8M",
        "-maxrate",
        "10M",
        "-bufsize",
        "16M",
        "-profile:v",
        "high",
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "-ar",
        "44100",
        "-pix_fmt",
        "yuv420p",
        "-movflags",
        "+faststart",
        "-y",
        output_path,
    ]);

    run_ffmpeg_command(args)
}

pub fn export_audio_only(input_path: &str, output_path: &str) -> Result<(), String> {
    let args = vec![
        "-i",
        input_path,
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "-ar",
        "44100",
        "-y",
        output_path,
    ];

    run_ffmpeg_command(args)
}

fn run_ffmpeg_command(args: Vec<&str>) -> Result<(), String> {
    run_ffmpeg_command_raw(args).map(|_| ())
}

fn run_ffmpeg_command_raw(args: Vec<&str>) -> Result<String, String> {
    let output = Command::new("binaries/ffmpeg")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg failed: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
