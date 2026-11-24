//! crates/iris-core/src/device.rs
//!
//! Camera hardware discovery, device classification (built-in vs external USB),
//! and dynamic resolution negotiation from Low (480p) to Ultra HD (4K).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    /// Built-in integrated camera on laptop / all-in-one desktop (PCI / eDP bridge)
    BuiltIn,
    /// External USB webcam plugged into laptop or desktop workstation
    ExternalUsb,
    /// High-fidelity sovereign simulated camera sensor for headless / virtualization
    VirtualHD,
}

impl DeviceKind {
    pub fn label(&self) -> &'static str {
        match self {
            DeviceKind::BuiltIn => "Built-in Camera",
            DeviceKind::ExternalUsb => "External USB Camera",
            DeviceKind::VirtualHD => "NovaVision Virtual HD Camera",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Resolution {
    /// 480p SD (640x480, 4:3 standard definition)
    Res480p,
    /// 720p HD (1280x720, 16:9 high definition)
    Res720p,
    /// 1080p FHD (1920x1080, 16:9 full high definition)
    Res1080p,
    /// 4K UHD (3840x2160, 16:9 ultra high definition)
    Res4K,
}

impl Resolution {
    pub const ALL: [Resolution; 4] = [
        Resolution::Res480p,
        Resolution::Res720p,
        Resolution::Res1080p,
        Resolution::Res4K,
    ];

    pub fn width(&self) -> u32 {
        match self {
            Resolution::Res480p => 640,
            Resolution::Res720p => 1280,
            Resolution::Res1080p => 1920,
            Resolution::Res4K => 3840,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Resolution::Res480p => 480,
            Resolution::Res720p => 720,
            Resolution::Res1080p => 1080,
            Resolution::Res4K => 2160,
        }
    }

    pub fn aspect_ratio_str(&self) -> &'static str {
        match self {
            Resolution::Res480p => "4:3",
            Resolution::Res720p | Resolution::Res1080p | Resolution::Res4K => "16:9",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Resolution::Res480p => "480p SD (640×480)",
            Resolution::Res720p => "720p HD (1280×720)",
            Resolution::Res1080p => "1080p FHD (1920×1080)",
            Resolution::Res4K => "4K UHD (3840×2160)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Resolution::Res480p => "480p",
            Resolution::Res720p => "720p",
            Resolution::Res1080p => "1080p",
            Resolution::Res4K => "4K",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    YUYV,
    MJPEG,
    RGB24,
    RGBA32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CameraDevice {
    pub id: String,
    pub name: String,
    pub device_node: String, // e.g. "/dev/video0"
    pub kind: DeviceKind,
    pub supported_resolutions: Vec<Resolution>,
    pub max_framerate: u32,
    pub is_default: bool,
}

impl CameraDevice {
    pub fn new(
        id: &str,
        name: &str,
        device_node: &str,
        kind: DeviceKind,
        supported_resolutions: Vec<Resolution>,
        max_framerate: u32,
        is_default: bool,
    ) -> Self {
        CameraDevice {
            id: id.to_string(),
            name: name.to_string(),
            device_node: device_node.to_string(),
            kind,
            supported_resolutions,
            max_framerate,
            is_default,
        }
    }

    /// Best available resolution on this sensor
    pub fn highest_resolution(&self) -> Resolution {
        self.supported_resolutions
            .iter()
            .copied()
            .max()
            .unwrap_or(Resolution::Res720p)
    }

    /// Low/Default resolution for fast viewfinder startup
    pub fn default_resolution(&self) -> Resolution {
        if self.supported_resolutions.contains(&Resolution::Res720p) {
            Resolution::Res720p
        } else if self.supported_resolutions.contains(&Resolution::Res1080p) {
            Resolution::Res1080p
        } else {
            self.supported_resolutions.first().copied().unwrap_or(Resolution::Res480p)
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceRegistry {
    pub devices: Vec<CameraDevice>,
    pub active_device_idx: usize,
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        let mut reg = DeviceRegistry {
            devices: Vec::new(),
            active_device_idx: 0,
        };
        reg.populate_standard_devices();
        reg
    }
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-populates realistic hardware devices mimicking a standard dual-camera
    /// laptop/desktop workstation setup:
    /// 1. Integrated HD Webcam (Built-in laptop camera, 720p/1080p)
    /// 2. External USB Video Device (Desktop workstation camera, up to 4K UHD)
    /// 3. Virtual Sovereign HD Sensor (Always available for testing and VM environments)
    pub fn populate_standard_devices(&mut self) {
        self.devices.clear();

        // 1. Built-in Laptop Webcam
        self.devices.push(CameraDevice::new(
            "builtin_0",
            "Integrated HD Camera (Built-in)",
            "/dev/video0",
            DeviceKind::BuiltIn,
            vec![Resolution::Res480p, Resolution::Res720p, Resolution::Res1080p],
            30,
            true,
        ));

        // 2. External USB Camera (Desktop / Peripheral)
        self.devices.push(CameraDevice::new(
            "usb_cam_1",
            "NovaCam Pro 4K (External USB)",
            "/dev/video1",
            DeviceKind::ExternalUsb,
            vec![
                Resolution::Res480p,
                Resolution::Res720p,
                Resolution::Res1080p,
                Resolution::Res4K,
            ],
            60,
            false,
        ));

        // 3. Virtual HD Camera (Headless / VM)
        self.devices.push(CameraDevice::new(
            "virtual_hd",
            "NovaVision Sovereign Sensor (Virtual)",
            "/dev/video_virt",
            DeviceKind::VirtualHD,
            vec![
                Resolution::Res480p,
                Resolution::Res720p,
                Resolution::Res1080p,
                Resolution::Res4K,
            ],
            60,
            false,
        ));

        self.active_device_idx = 0;
    }

    pub fn active_device(&self) -> Option<&CameraDevice> {
        self.devices.get(self.active_device_idx)
    }

    pub fn select_device_by_id(&mut self, id: &str) -> bool {
        if let Some(pos) = self.devices.iter().position(|d| d.id == id) {
            self.active_device_idx = pos;
            true
        } else {
            false
        }
    }

    pub fn cycle_next_camera(&mut self) -> Option<&CameraDevice> {
        if !self.devices.is_empty() {
            self.active_device_idx = (self.active_device_idx + 1) % self.devices.len();
            self.active_device()
        } else {
            None
        }
    }

    pub fn register_device(&mut self, device: CameraDevice) {
        if let Some(pos) = self.devices.iter().position(|d| d.id == device.id) {
            self.devices[pos] = device;
        } else {
            self.devices.push(device);
        }
    }

    pub fn unregister_device(&mut self, id: &str) {
        self.devices.retain(|d| d.id != id);
        if self.active_device_idx >= self.devices.len() && !self.devices.is_empty() {
            self.active_device_idx = self.devices.len() - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_enumeration_and_classification() {
        let reg = DeviceRegistry::default();
        assert_eq!(reg.devices.len(), 3);

        let builtin = &reg.devices[0];
        assert_eq!(builtin.kind, DeviceKind::BuiltIn);
        assert_eq!(builtin.device_node, "/dev/video0");
        assert!(builtin.is_default);

        let external = &reg.devices[1];
        assert_eq!(external.kind, DeviceKind::ExternalUsb);
        assert_eq!(external.highest_resolution(), Resolution::Res4K);
    }

    #[test]
    fn test_resolution_dimensions_and_aspect_ratio() {
        assert_eq!(Resolution::Res480p.width(), 640);
        assert_eq!(Resolution::Res480p.height(), 480);
        assert_eq!(Resolution::Res480p.aspect_ratio_str(), "4:3");

        assert_eq!(Resolution::Res1080p.width(), 1920);
        assert_eq!(Resolution::Res1080p.height(), 1080);
        assert_eq!(Resolution::Res1080p.aspect_ratio_str(), "16:9");
    }

    #[test]
    fn test_camera_switching_cycle() {
        let mut reg = DeviceRegistry::default();
        assert_eq!(reg.active_device_idx, 0);

        reg.cycle_next_camera();
        assert_eq!(reg.active_device_idx, 1);
        assert_eq!(reg.active_device().unwrap().id, "usb_cam_1");

        reg.cycle_next_camera();
        assert_eq!(reg.active_device_idx, 2);

        reg.cycle_next_camera();
        assert_eq!(reg.active_device_idx, 0);
    }
}
