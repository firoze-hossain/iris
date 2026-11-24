# Iris Camera & Video Studio 📷🎥

**Iris** (*Ἶρις* / Camera Iris, Lens & Vision) is the sovereign, cross-platform pure-Rust default Camera & Video Studio application for **NovaOS**.

Synthesizing the modern visual layout and interaction design of **Ubuntu Camera (GNOME Snapshot)** with sovereign cross-platform capture backends (Linux V4L2, macOS AVFoundation, Windows MediaFoundation, and NovaOS ring-3 kernel video capture IPC), Iris provides live viewfinder video feeds, dynamic built-in laptop and external desktop webcam detection, dynamic resolution switching from 480p SD up to 4K UHD, countdown timers, photo snapshots, video recording sessions, and media library management.

---

## 🌟 Key Features

### 1. 🪟 Ubuntu Camera (GNOME Snapshot) UI
- **Live Viewfinder Canvas**: Real-time camera feed scaling with aspect-ratio letterboxing.
- **Iconic Shutter / Record Button**:
  - **Photo Mode**: Double-ring white button with solid white center.
  - **Video Idle Mode**: Outer white ring with vibrant red center.
  - **Video Recording Mode**: Outer white ring with red rounded square stop button.
- **Dual Mode Switcher Pill**: Instant toggle between Photo (`📷`) and Video (`📹`) modes.
- **Countdown Timer**: Configurable countdown delays (`Off`, `3s`, `5s`, `10s`) with on-screen visual overlay (`3... 2... 1... 📸`) and snapshot screen flash illumination.
- **Composition Grids**: 3×3 Rule of Thirds, Golden Ratio, and Center Crosshair guide overlays.
- **Selfie Mirror Mode**: Horizontal flip transformation for natural webcam front-facing orientation.

### 2. 🔌 Dynamic Hardware Detection & Classification
- **Built-in Laptop Webcams**: Automatically detects integrated laptop cameras (PCI/eDP/MIPI).
- **External USB Webcams**: Hotplug discovery of desktop USB cameras and high-end external video peripherals.
- **Sovereign Virtual Sensor**: Always-available high-fidelity simulated camera with SMPTE test bars and live timestamps for virtual machines, cloud instances, and CI test pipelines.
- **One-Click Camera Cycling**: Seamlessly switch between connected cameras via header pill (`[ Integrated HD Camera ] 🔄`).

### 3. 📐 Dynamic Resolution Hierarchy (Low to 4K)
- **480p SD**: 640×480 (4:3 aspect ratio) standard definition for lightweight streaming.
- **720p HD**: 1280×720 (16:9 aspect ratio) high definition balanced default.
- **1080p FHD**: 1920×1080 (16:9 aspect ratio) full high definition for crisp photography.
- **4K UHD**: 3840×2160 (16:9 aspect ratio) ultra high definition for studio-grade capture.

### 4. 🎞️ Photo Snapshot & Video Recording Pipeline
- **Photo Snapshots**: Lossless Netpbm PPM (P6) binary export saved directly to `~/Pictures/Camera/`.
- **Video Recording Sessions**: Frame counter, live elapsed recording timer (`● REC 00:02:15`), bitrate calculation, and raw video frame persistence in `~/Videos/Camera/`.
- **Media Library Gallery**: Instant thumbnail preview of the most recent capture with total item count and metadata telemetry.

---

## 🏗️ Workspace Architecture

```
iris/
├── Cargo.toml
├── README.md
├── crates/
│   ├── iris-core/          # Pure domain logic: device registry, resolution hierarchy, frame math, capture engine, gallery
│   │   ├── src/
│   │   │   ├── device.rs   # CameraDevice, DeviceKind (BuiltIn vs ExternalUsb), Resolution (480p..4K), DeviceRegistry
│   │   │   ├── frame.rs    # VideoFrame, horizontal mirror flip, composition grids, Netpbm PPM export
│   │   │   ├── capture.rs  # CaptureEngine, countdown timers, flash animation, VideoRecordingSession
│   │   │   ├── gallery.rs  # MediaItem, MediaKind (Photo vs Video), GalleryRepo
│   │   │   ├── config.rs   # IrisConfig, CameraMode, TimerDuration, save directories
│   │   │   ├── hub.rs      # CameraHub reactive state coordinator
│   │   │   └── lib.rs      # Re-exports
│   │   └── Cargo.toml
│   ├── iris-ui/            # 2D software rasterizer & Ubuntu Snapshot visual components
│   │   ├── src/
│   │   │   ├── geom.rs     # Rect, Point, Color ARGB, camera palette
│   │   │   ├── font.rs     # Bitmap typography, camera icons (📷, 📹, ⏱, ≡, 🔄, ●, ■)
│   │   │   ├── draw.rs     # Surface trait, letterboxed video blitter, circular shutter buttons
│   │   │   ├── views.rs    # Viewfinder, top bar, floating bottom control bar, settings modal
│   │   │   └── lib.rs      # Headless test verification
│   │   └── Cargo.toml
│   ├── iris-platform/      # Cross-platform video capture backend hooks
│   │   ├── src/
│   │   │   └── lib.rs      # Linux V4L2 (/sys/class/video4linux), macOS AVFoundation, Windows MediaFoundation, NovaOS Ring-3
│   │   └── Cargo.toml
│   └── iris-app/           # Standalone desktop CLI application & QA qualification harness
│       ├── src/
│       │   └── main.rs     # CLI commands, --selftest, --benchmark, --status, --photo, --record
│       └── Cargo.toml
└── novaos/                 # NovaOS ring-3 bare-metal integration
    ├── ffi.rs              # System call wrappers (sys_write, sys_yield)
    ├── iris.rs             # Pure #[no_std] #[no_main] ring-3 binary
    └── build.sh            # Cross-compiles CAM.ELF & generates CAM.PKG fixtures
```

---

## 🚀 Building & Running

### Cross-Platform Desktop CLI (Linux / macOS / Windows)
```bash
# Run autonomous qualification suite (26 tests across 7 subsystems)
cargo run --release -p iris-app -- --selftest

# Benchmark multi-resolution pipeline (480p to 4K)
cargo run --release -p iris-app -- --benchmark

# View camera hardware status and telemetry
cargo run --release -p iris-app -- --status

# List all detected built-in and external USB cameras
cargo run --release -p iris-app -- --list-devices

# Capture a snapshot to PPM image
cargo run --release -p iris-app -- --photo test_snapshot.ppm
```

### NovaOS Bare-Metal Ring-3 Executable
```bash
# Build i686 ELF32 binary and install to NovaOS fixtures
./novaos/build.sh
```

---

## 📜 License
MIT OR Apache-2.0
