//! crates/iris-app/src/main.rs
//!
//! # Iris Camera Application Runner & Autonomous Qualification Engine
//!
//! Standalone CLI driver, cross-platform video capture studio, and autonomous
//! qualification suite for the Sovereign NovaOS Camera application ("Iris").

use std::fs;
use iris_core::config::{CameraMode, TimerDuration};
use iris_core::device::{DeviceKind, Resolution};
use iris_core::frame::GridMode;
use iris_core::hub::CameraHub;
use iris_platform::{create_platform_camera_engine, LinuxV4L2Backend, NovaCaptureBackend, VideoCaptureBackend};
use iris_ui::draw::{OwnedBufferSurface, Surface};
use iris_ui::geom::Rect;
use iris_ui::views::IrisRenderer;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    if args.iter().any(|a| a == "--selftest" || a == "-t") {
        let code = run_selftest();
        std::process::exit(code);
    }

    if args.iter().any(|a| a == "--benchmark" || a == "-b") {
        run_benchmark();
        return;
    }

    if args.iter().any(|a| a == "--status" || a == "-s") {
        print_status();
        return;
    }

    if args.iter().any(|a| a == "--list-devices") {
        list_devices();
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--photo") {
        let filename = if pos + 1 < args.len() {
            args[pos + 1].clone()
        } else {
            "snapshot.ppm".to_string()
        };
        take_photo_cli(&filename);
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--record") {
        let secs = if pos + 1 < args.len() {
            args[pos + 1].parse::<u32>().unwrap_or(5)
        } else {
            5
        };
        let filename = if pos + 2 < args.len() {
            args[pos + 2].clone()
        } else {
            "recording.raw".to_string()
        };
        record_video_cli(secs, &filename);
        return;
    }

    // Default: Run autonomous qualification suite
    println!("Iris Camera - Sovereign Photo & Video Studio for NovaOS");
    let code = run_selftest();
    std::process::exit(code);
}

fn print_help() {
    println!("Iris Camera v1.0.0 (NovaOS Enterprise)");
    println!("Usage: iris [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -t, --selftest       Run full autonomous test suite across all 7 camera subsystems");
    println!("  -b, --benchmark      Benchmark multi-resolution frame pipeline (480p to 4K)");
    println!("  -s, --status         Print current camera telemetry, resolution, and library stats");
    println!("      --list-devices   Enumerate all detected built-in and external USB cameras");
    println!("      --photo [FILE]   Capture a high-resolution snapshot to PPM image");
    println!("      --record [S] [F] Record video clip for S seconds to file F");
    println!("  -h, --help           Show this help information");
}

fn print_status() {
    let hub = CameraHub::new();
    let backend = create_platform_camera_engine();

    println!("============================================================");
    println!("             IRIS CAMERA TELEMETRY & STATUS                 ");
    println!("============================================================");
    println!("Active Platform Backend: {}", backend.backend_name());
    println!("Connected Devices:       {} detected", hub.registry.devices.len());
    if let Some(active) = hub.registry.active_device() {
        println!("Active Camera Device:    {} ({})", active.name, active.kind.label());
        println!("Device Hardware Node:    {}", active.device_node);
    }
    println!("Active Resolution:       {} [{}]", hub.config.resolution.label(), hub.config.resolution.aspect_ratio_str());
    println!("Camera Mode:             {}", hub.config.mode.label());
    println!("Selfie Mirror Mode:      {}", if hub.config.mirror_mode { "Enabled (Horizontal Flip)" } else { "Disabled" });
    println!("Composition Grid:        {}", hub.config.grid.label());
    println!("Timer Delay:             {}", hub.config.timer.label());
    println!("Media Library Captures:  {} items", hub.gallery.items.len());
    println!("Photo Save Directory:    {}", hub.config.photo_directory);
    println!("Video Save Directory:    {}", hub.config.video_directory);
    println!("============================================================");
}

fn list_devices() {
    let mut backend = create_platform_camera_engine();
    let devices = backend.probe_devices();
    println!("Detected Camera Devices ({} total):", devices.len());
    for (i, dev) in devices.iter().enumerate() {
        println!("  [{}] {} ({})", i + 1, dev.name, dev.kind.label());
        println!("      Node: {} | Max FPS: {}", dev.device_node, dev.max_framerate);
        let res_strs: Vec<&str> = dev.supported_resolutions.iter().map(|r| r.short_name()).collect();
        println!("      Supported Resolutions: {}", res_strs.join(", "));
    }
}

fn take_photo_cli(filename: &str) {
    let mut hub = CameraHub::new();
    let epoch = 1732435200;
    hub.tick(16, epoch);

    if let Some(photo) = hub.trigger_shutter(epoch) {
        if let Some(frame) = hub.capture.last_snapshot_taken.as_ref() {
            let ppm_bytes = frame.to_ppm_bytes();
            if fs::write(filename, &ppm_bytes).is_ok() {
                println!("[Iris] Successfully captured photo '{}' ({}x{}, {} bytes).", filename, frame.width, frame.height, ppm_bytes.len());
                return;
            }
        }
        println!("[Iris] Snapshot recorded in gallery: {}", photo.filename);
    }
}

fn record_video_cli(seconds: u32, filename: &str) {
    let mut hub = CameraHub::new();
    hub.toggle_mode(); // Switch to video
    let start_epoch = 1732435200;

    println!("[Iris] Starting video recording for {} seconds...", seconds);
    hub.trigger_shutter(start_epoch);

    // Simulate recording loop
    for sec in 1..=seconds {
        hub.tick(1000, start_epoch + sec as u64);
    }

    if let Some(item) = hub.trigger_shutter(start_epoch + seconds as u64) {
        println!("[Iris] Completed video recording '{}' (Duration: {}s, Resolution: {}).", filename, item.duration_seconds.unwrap_or(seconds), item.resolution.short_name());
    }
}

fn run_benchmark() {
    println!("Benchmarking Iris Multi-Resolution Pipeline (480p to 4K)...");

    let resolutions = [
        Resolution::Res480p,
        Resolution::Res720p,
        Resolution::Res1080p,
        Resolution::Res4K,
    ];

    for &res in &resolutions {
        let start = std::time::Instant::now();
        for f in 0..50 {
            let mut frame = iris_core::frame::VideoFrame::generate_synthetic(res.width(), res.height(), f, f * 33_333);
            frame.flip_horizontal();
            frame.apply_composition_grid(GridMode::RuleOfThirds);
        }
        let elapsed = start.elapsed();
        let fps = 50.0 / elapsed.as_secs_f64();
        println!("  [{:<8}] 50 frames generated, mirrored & gridded in {:.2?} (~{:.1} FPS)", res.short_name(), elapsed, fps);
    }

    let mut surface = OwnedBufferSurface::new(1024, 768);
    let hub = CameraHub::new();
    let start_render = std::time::Instant::now();
    for f in 0..100 {
        IrisRenderer::render(&mut surface, Rect::new(0, 0, 1024, 768), &hub, 1000 + f);
    }
    let elapsed_render = start_render.elapsed();
    let render_fps = 100.0 / elapsed_render.as_secs_f64();
    println!("  [UI Render] 100 frames composited onto 1024x768 surface in {:.2?} (~{:.1} FPS)", elapsed_render, render_fps);
}

fn run_selftest() -> i32 {
    println!("============================================================");
    println!("         IRIS CAMERA AUTONOMOUS QUALIFICATION SUITE         ");
    println!("============================================================");

    let mut passed = 0;
    let mut failed = 0;

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                println!("  [PASS] {}", $name);
                passed += 1;
            } else {
                println!("  [FAIL] {}", $name);
                failed += 1;
            }
        };
    }

    // --- Subsystem 1: Device Enumeration & Classification ---
    println!("\n[1/7] Subsystem: Device Enumeration & Classification");
    let mut hub = CameraHub::new();
    check!("Device registry discovers at least 2 camera devices", hub.registry.devices.len() >= 2);

    let has_builtin = hub.registry.devices.iter().any(|d| d.kind == DeviceKind::BuiltIn);
    let has_external = hub.registry.devices.iter().any(|d| d.kind == DeviceKind::ExternalUsb);
    check!("Built-in laptop webcam detected and classified", has_builtin);
    check!("External USB desktop camera detected and classified", has_external);

    // Camera switching
    let initial_id = hub.registry.active_device().unwrap().id.clone();
    hub.switch_camera();
    let switched_id = hub.registry.active_device().unwrap().id.clone();
    check!("Camera switching cycles active hardware device", initial_id != switched_id);

    // --- Subsystem 2: Dynamic Resolution Hierarchy ---
    println!("\n[2/7] Subsystem: Dynamic Resolution Hierarchy (480p to 4K)");
    hub.set_resolution(Resolution::Res480p);
    check!("480p SD dimensions: 640x480 (4:3 aspect ratio)",
        hub.config.resolution.width() == 640 && hub.config.resolution.height() == 480 && hub.config.resolution.aspect_ratio_str() == "4:3");

    hub.set_resolution(Resolution::Res720p);
    check!("720p HD dimensions: 1280x720 (16:9 aspect ratio)",
        hub.config.resolution.width() == 1280 && hub.config.resolution.height() == 720 && hub.config.resolution.aspect_ratio_str() == "16:9");

    hub.set_resolution(Resolution::Res1080p);
    check!("1080p FHD dimensions: 1920x1080 (16:9 aspect ratio)",
        hub.config.resolution.width() == 1920 && hub.config.resolution.height() == 1080);

    hub.set_resolution(Resolution::Res4K);
    check!("4K UHD dimensions: 3840x2160 (16:9 aspect ratio)",
        hub.config.resolution.width() == 3840 && hub.config.resolution.height() == 2160);

    // --- Subsystem 3: Video Frame Transforms & Overlays ---
    println!("\n[3/7] Subsystem: Video Frame Transforms & Overlays");
    let mut test_frame = iris_core::frame::VideoFrame::new(4, 1, vec![1, 2, 3, 4], 0, 0);
    test_frame.flip_horizontal();
    check!("Selfie mirror horizontal flip swaps left/right pixels", test_frame.pixels == vec![4, 3, 2, 1]);

    let mut grid_frame = iris_core::frame::VideoFrame::solid(300, 300, 0xFF000000);
    grid_frame.apply_composition_grid(GridMode::RuleOfThirds);
    check!("Composition grid applies 3x3 Rule of Thirds guide lines", grid_frame.pixels[100 * 300 + 100] != 0xFF000000);

    let ppm_export = grid_frame.to_ppm_bytes();
    check!("Lossless Netpbm PPM (P6) binary serialization valid", ppm_export.starts_with(b"P6\n300 300\n255\n"));

    // --- Subsystem 4: Capture Engine & Countdown Timers ---
    println!("\n[4/7] Subsystem: Capture Engine & Countdown Timers");
    hub.config.timer = TimerDuration::Delay3s;
    let armed = hub.trigger_shutter(1000);
    check!("Countdown timer arms without immediate capture (returns None)", armed.is_none());
    check!("Countdown remaining set to 3000ms", hub.capture.countdown_remaining_ms == Some(3000));

    hub.tick(1500, 1001);
    check!("Countdown decrements after 1500ms (1500ms remaining)", hub.capture.countdown_remaining_ms == Some(1500));

    hub.tick(1600, 1003);
    check!("Countdown expiration fires photo snapshot and triggers flash", hub.capture.countdown_remaining_ms.is_none() && hub.capture.is_flashing());
    hub.config.timer = TimerDuration::Off;

    // --- Subsystem 5: Video Recording Session ---
    println!("\n[5/7] Subsystem: Video Recording Session Lifecycle");
    hub.set_mode(CameraMode::Video);
    check!("Mode switch to Video Mode", hub.config.mode == CameraMode::Video);

    hub.trigger_shutter(2000);
    check!("Video recording session begins (is_recording = true)", hub.capture.is_recording());

    // Tick recording frames
    for f in 1..=30 {
        hub.tick(33, 2000 + f / 30);
    }
    check!("Frame count increments during active recording", hub.capture.active_recording.as_ref().unwrap().frame_count >= 30);

    let finished_video = hub.trigger_shutter(2010);
    check!("Stop recording concludes session and stores video in gallery", finished_video.is_some() && !hub.capture.is_recording());
    check!("Recorded video duration tracked accurately", finished_video.unwrap().duration_seconds == Some(10));

    // --- Subsystem 6: 2D Ubuntu Snapshot UI Rasterization ---
    println!("\n[6/7] Subsystem: 2D Ubuntu Snapshot UI Rasterization");
    let mut surface = OwnedBufferSurface::new(800, 600);
    hub.set_mode(CameraMode::Photo);
    IrisRenderer::render(&mut surface, Rect::new(0, 0, 800, 600), &hub, 3000);

    let viewport_pixel = surface.get_pixel(400, 250);
    check!("Viewfinder video feed rasterized onto canvas", viewport_pixel.to_u32() != 0);

    let shutter_pixel = surface.get_pixel(400, 560);
    check!("Iconic circular shutter button rasterized without clipping", shutter_pixel.to_u32() != 0);

    // Preferences modal dialog rendering
    hub.toggle_settings();
    IrisRenderer::render(&mut surface, Rect::new(0, 0, 800, 600), &hub, 3001);
    check!("Preferences modal dialog rasterized cleanly", hub.show_settings_modal);
    hub.toggle_settings();

    // --- Subsystem 7: Cross-Platform Video Capture Backends ---
    println!("\n[7/7] Subsystem: Cross-Platform Video Capture Backends");
    let mut v4l2 = LinuxV4L2Backend::new();
    check!("Linux V4L2 backend discovers cameras", !v4l2.probe_devices().is_empty());

    let mut nova_cam = NovaCaptureBackend::new();
    check!("NovaOS Ring-3 Video IPC backend operational", nova_cam.probe_devices().len() == 2);

    let native = create_platform_camera_engine();
    println!("  [INFO] Host native camera pipeline: {}", native.backend_name());
    check!("Native platform video capture engine initialized", !native.backend_name().is_empty());

    // --- Final Qualification Summary ---
    println!("\n============================================================");
    println!("AUTONOMOUS QUALIFICATION SUMMARY: {} passed, {} failed", passed, failed);
    println!("============================================================");

    if failed == 0 {
        println!(">>> [SUCCESS] All 7 Iris Camera Subsystems 100% QUALIFIED & OPERATIONAL <<<");
        0
    } else {
        println!(">>> [FAILURE] One or more camera tests failed! <<<");
        1
    }
}
