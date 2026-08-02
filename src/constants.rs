use wgpu::{PresentMode, TextureFormat};

pub const WINDOW_TITLE: &str = "game";
pub const WINDOW_WIDTH: u32 = 950;
pub const WINDOW_HEIGHT: u32 = 950;

pub const CAMERA_FOV: f32 = std::f32::consts::PI / 4.0;
pub const CAMERA_NEAR: f32 = 0.1;
pub const CAMERA_FAR: f32 = 100.0;

pub const INITIAL_TRANSLATION: [f32; 4] = [0.0, 0.0, 4.5, 0.0];
pub const INITIAL_ROTATION: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

pub const CAMERA_INITIAL_DISTANCE: f32 = 6.0;
pub const CAMERA_MIN_DISTANCE: f32 = 1.0;
pub const CAMERA_MAX_DISTANCE: f32 = 50.0;
pub const CAMERA_ZOOM_SPEED: f32 = 1.0;
pub const CAMERA_ORBIT_SPEED: f32 = 0.008;
pub const CAMERA_PAN_SPEED: f32 = 0.005;

pub const DEFAULT_MODEL_PATH: &str = "null.obj";
pub const DEFAULT_TEXTURE_PATH: &str = "null.png";
pub const NULL_TEXTURE_PATH: &str = "tex/null.png";

pub const DEFAULT_VERTEX_HALF_SIZE: f32 = 0.5;

pub const MODEL_AUTO_SCALE_EXTENT: f32 = 3.0;
pub const DEFAULT_BASE_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0];

pub const UI_PANEL_FILL: [u8; 3] = [38, 38, 42];
pub const UI_BG_FILL: [u8; 3] = [48, 48, 54];
pub const UI_INACTIVE_FILL: [u8; 3] = [58, 58, 64];
pub const UI_HOVERED_FILL: [u8; 3] = [68, 68, 76];
pub const UI_ACTIVE_FILL: [u8; 3] = [78, 78, 86];
pub const UI_ACCENT: [u8; 3] = [61, 110, 245];

pub const PRESENT_MODE: PresentMode = PresentMode::Fifo;
pub const FRAME_LATENCY: u32 = 2;
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
pub const DEFAULT_SAMPLE_COUNT: u32 = 1;

pub const LIGHT_DIR: [f32; 4] = [0.5, 0.7, 0.5, 0.0];
