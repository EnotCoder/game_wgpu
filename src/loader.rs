use std::path::Path;

use crate::models::{load_gltf, load_obj, load_stl, normalize_model, ModelObj};

#[derive(Debug, Clone, PartialEq)]
pub enum ModelFormat {
    Obj,
    Gltf,
    Glb,
    Stl,
    Unknown,
}

impl ModelFormat {
    pub fn from_path(path: &str) -> Self {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "obj" => ModelFormat::Obj,
            "gltf" => ModelFormat::Gltf,
            "glb" => ModelFormat::Glb,
            "stl" => ModelFormat::Stl,
            _ => ModelFormat::Unknown,
        }
    }
}

pub fn load_model(path: &str) -> Result<ModelObj, String> {
    let format = ModelFormat::from_path(path);
    let mut model = match format {
        ModelFormat::Obj => load_obj(path),
        ModelFormat::Gltf | ModelFormat::Glb => load_gltf(path),
        ModelFormat::Stl => load_stl(path),
        ModelFormat::Unknown => Err(format!("Unknown model format: {}", path)),
    }?;
    normalize_model(&mut model);
    Ok(model)
}
