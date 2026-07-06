use wgpu::{PresentMode, TextureFormat};

pub const WINDOW_TITLE: &str = "game";
pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;

pub const CAMERA_FOV: f32 = std::f32::consts::PI / 4.0;
pub const CAMERA_NEAR: f32 = 0.1;
pub const CAMERA_FAR: f32 = 100.0;

pub const INITIAL_TRANSLATION: [f32; 4] = [0.0, 0.0, 4.5, 0.0];
pub const INITIAL_ROTATION: [f32; 4] = [-0.2, 0.0, 0.0, 0.0];
pub const FON_TRANSLATION_BASE: [f32; 4] = [0.0, 0.0, 15.0, 0.0];

pub const SCROLL_STEP: f32 = 0.5;
pub const ZOOM_MAX: f32 = 5.0;
pub const ZOOM_MIN: f32 = -2.0;

pub const DEFAULT_MODEL_PATH: &str = "null.obj";
pub const DEFAULT_TEXTURE_PATH: &str = "null.png";
pub const NULL_TEXTURE_PATH: &str = "tex/null.png";
pub const FON_MODEL_PATH: &str = "models/fon.obj";
pub const FON_TEXTURE_PATH: &str = "tex/fon_texture.png";

pub const DEFAULT_VERTEX_HALF_SIZE: f32 = 0.5;

pub const EGUI_WINDOW_ROUNDING: f32 = 5.0;

pub const PRESENT_MODE: PresentMode = PresentMode::Fifo;
pub const FRAME_LATENCY: u32 = 2;
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
pub const DEFAULT_SAMPLE_COUNT: u32 = 1;

pub const LIGHT_DIR: [f32; 4] = [0.5, 0.7, 0.5, 0.0];

pub const EGUI_PANEL_POS: [f32; 2] = [10.0, 10.0];
pub const EGUI_PANEL_SIZE: [f32; 2] = [280.0, 400.0];
pub const EGUI_INFO_POS: [f32; 2] = [580.0, 10.0];
pub const EGUI_INFO_SIZE: [f32; 2] = [400.0, 400.0];
pub const EGUI_EFFECT_POS: [f32; 2] = [580.0, 10.0];
pub const EGUI_EFFECT_SIZE: [f32; 2] = [400.0, 400.0];
