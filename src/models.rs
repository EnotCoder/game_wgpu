use std::fs::File;
use std::io::{BufRead, BufReader};
use wgpu::{util::{DeviceExt, BufferInitDescriptor}, *};

use crate::constants::*;
use crate::texture::LoadedTexture;
use crate::Uniforms;
use crate::Vertex;

pub struct ModelObj {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn load_obj_simple(path: &str) -> Result<ModelObj, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut positions = Vec::new();
    let mut tex_coords = Vec::new();
    let mut face_indices = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {
                let x = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let z = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                positions.push([x, y, z]);
            }
            "vt" => {
                let u = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let v = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                tex_coords.push([u, v]);
            }
            "f" => {
                for part in parts.iter().skip(1) {
                    let idx_parts: Vec<&str> = part.split('/').collect();
                    let pos_idx = idx_parts
                        .first()
                        .and_then(|s| s.parse::<usize>().ok())
                        .map(|i| i - 1)
                        .unwrap_or(0);

                    let tex_idx = if idx_parts.len() > 1 && !idx_parts[1].is_empty() {
                        idx_parts[1].parse::<usize>().ok().map(|i| i - 1)
                    } else {
                        None
                    };

                    face_indices.push((pos_idx, tex_idx));
                }
            }
            _ => {}
        }
    }

    let mut normals_per_vertex: Vec<Vec<[f32; 3]>> = vec![Vec::new(); positions.len()];
    for chunk in face_indices.chunks(3) {
        if chunk.len() < 3 {
            continue;
        }
        let p0 = positions[chunk[0].0];
        let p1 = positions[chunk[1].0];
        let p2 = positions[chunk[2].0];
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        let len = f32::sqrt(nx * nx + ny * ny + nz * nz);
        let normal = if len > 0.0 {
            [nx / len, ny / len, nz / len]
        } else {
            [0.0, 0.0, 1.0]
        };
        for &(pos_idx, _) in chunk {
            normals_per_vertex[pos_idx].push(normal);
        }
    }

    let mut averaged_normals = vec![[0.0, 0.0, 0.0]; positions.len()];
    for (i, norms) in normals_per_vertex.iter().enumerate() {
        if norms.is_empty() {
            averaged_normals[i] = [0.0, 0.0, 1.0];
            continue;
        }
        let sum: [f32; 3] =
            norms.iter()
                .fold([0.0, 0.0, 0.0], |acc, n| [acc[0] + n[0], acc[1] + n[1], acc[2] + n[2]]);
        let count = norms.len() as f32;
        let avg = [sum[0] / count, sum[1] / count, sum[2] / count];
        let len = f32::sqrt(avg[0] * avg[0] + avg[1] * avg[1] + avg[2] * avg[2]);
        averaged_normals[i] = if len > 0.0 {
            [avg[0] / len, avg[1] / len, avg[2] / len]
        } else {
            [0.0, 0.0, 1.0]
        };
    }

    let mut vertices = Vec::with_capacity(face_indices.len());
    for (pos_idx, tex_idx_opt) in face_indices {
        let pos = positions[pos_idx];
        let normal = averaged_normals[pos_idx];
        let tex = tex_idx_opt
            .filter(|&idx| idx < tex_coords.len())
            .map(|idx| tex_coords[idx])
            .unwrap_or([0.0, 0.0]);

        vertices.push(Vertex {
            position: [pos[0], pos[1], pos[2]],
            normal,
            tex_coord: tex,
        });
    }

    let indices: Vec<u32> = (0..vertices.len() as u32).collect();
    Ok(ModelObj { vertices, indices })
}

fn default_vertices() -> Vec<Vertex> {
    let h = DEFAULT_VERTEX_HALF_SIZE;
    vec![
        Vertex { position: [-h, h, 0.0], normal: [0.0, 0.0, 1.0], tex_coord: [0.0, 0.0] },
        Vertex { position: [-h, -h, 0.0], normal: [0.0, 0.0, 1.0], tex_coord: [0.0, 1.0] },
        Vertex { position: [h, -h, 0.0], normal: [0.0, 0.0, 1.0], tex_coord: [1.0, 1.0] },
        Vertex { position: [h, -h, 0.0], normal: [0.0, 0.0, 1.0], tex_coord: [1.0, 1.0] },
        Vertex { position: [h, h, 0.0], normal: [0.0, 0.0, 1.0], tex_coord: [1.0, 0.0] },
        Vertex { position: [-h, h, 0.0], normal: [0.0, 0.0, 1.0], tex_coord: [0.0, 0.0] },
    ]
}

pub struct ModelInstance {
    #[allow(dead_code)]
    pub vertices: Vec<Vertex>,
    #[allow(dead_code)]
    pub indices: Vec<u16>,
    pub translation: [f32; 4],
    pub translation_base: [f32; 4],
    pub rotation: [f32; 4],
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
    pub texture_bind_group: BindGroup,
}

impl ModelInstance {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: &str,
        device: &Device,
        queue: &Queue,
        translation: [f32; 4],
        translation_base: [f32; 4],
        rotation: [f32; 4],
        projection: [f32; 16],
        texture_path: &str,
    ) -> Self {
        let (vertices, indices_u32) = match load_obj_simple(path) {
            Ok(model) => {
                println!(
                    "Loaded model: {} vertices, {} indices",
                    model.vertices.len(),
                    model.indices.len()
                );
                (model.vertices, model.indices)
            }
            Err(e) => {
                eprintln!("Failed to load model: {}", e);
                (default_vertices(), (0..6).collect())
            }
        };

        let indices: Vec<u16> = indices_u32.iter().map(|&i| i as u16).collect();
        let index_count = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let init_uniforms = Uniforms {
            translation,
            rotation,
            projection,
            use_texture: 1,
            _padding0: [0.0; 3],
            light_dir: LIGHT_DIR,
        };
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Model Uniform Buffer"),
            contents: bytemuck::cast_slice(&[init_uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Model Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Model Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let loaded_texture =
            LoadedTexture::from_path(device, queue, texture_path, "model_texture")
                .unwrap_or_else(|_| {
                    LoadedTexture::from_path(device, queue, NULL_TEXTURE_PATH, "null_texture")
                        .expect("Cannot continue without texture")
                });

        let tex_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &tex_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&loaded_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&loaded_texture.sampler),
                },
            ],
        });

        Self {
            vertices,
            indices,
            translation,
            translation_base,
            rotation,
            vertex_buffer,
            index_buffer,
            index_count,
            uniform_buffer,
            bind_group,
            texture_bind_group,
        }
    }

    pub fn update_transform(&self, queue: &Queue, projection: [f32; 16], use_texture: i32) {
        let uniforms = Uniforms {
            translation: self.translation,
            rotation: self.rotation,
            projection,
            use_texture,
            _padding0: [0.0; 3],
            light_dir: LIGHT_DIR,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}
