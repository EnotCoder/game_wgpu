use std::fs::File;
use std::io::Read;
use wgpu::{util::{DeviceExt, BufferInitDescriptor}, *};

use crate::constants::*;
use crate::texture::LoadedTexture;
use crate::Uniforms;
use crate::Vertex;

pub struct ModelObj {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn load_obj(path: &str) -> Result<ModelObj, String> {
    let (models, _materials) = tobj::load_obj(path, &tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
    }).map_err(|e| format!("OBJ error: {}", e))?;

    let mesh = &models.first().ok_or("No meshes in OBJ file")?.mesh;

    let mut vertices = Vec::with_capacity(mesh.indices.len());
    for &idx in &mesh.indices {
        let i = idx as usize;
        let pos = [mesh.positions[i * 3], mesh.positions[i * 3 + 1], mesh.positions[i * 3 + 2]];
        let normal = if !mesh.normals.is_empty() {
            [mesh.normals[i * 3], mesh.normals[i * 3 + 1], mesh.normals[i * 3 + 2]]
        } else {
            [0.0, 0.0, 0.0]
        };
        let tex = if !mesh.texcoords.is_empty() {
            [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]]
        } else {
            [0.0, 0.0]
        };
        vertices.push(Vertex { position: pos, normal, tex_coord: tex });
    }

    if mesh.normals.is_empty() {
        compute_flat_normals(&mut vertices);
    }

    let indices: Vec<u32> = (0..vertices.len() as u32).collect();
    println!("Loaded OBJ: {} vertices, {} triangles", vertices.len(), indices.len() / 3);
    Ok(ModelObj { vertices, indices })
}

pub fn load_gltf(path: &str) -> Result<ModelObj, String> {
    let (document, buffers, _images) = gltf::import(path).map_err(|e| format!("glTF error: {}", e))?;

    let mut vertices = Vec::new();
    let mut loaded_any = false;

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader.read_positions()
                .ok_or("No positions in glTF primitive")?
                .collect();
            let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(|n| n.collect());
            let texcoords: Option<Vec<[f32; 2]>> =
                reader.read_tex_coords(0).map(|t| t.into_f32().collect());
            let indices: Vec<u32> = reader.read_indices()
                .ok_or("No indices in glTF primitive")?
                .into_u32()
                .collect();

            let has_normals = normals.is_some();
            let norms = normals.as_ref();
            let texs = texcoords.as_ref();
            let base = vertices.len();

            for &idx in &indices {
                let i = idx as usize;
                let p = positions[i];
                let n = if has_normals { norms.unwrap()[i] } else { [0.0, 0.0, 0.0] };
                let t = texs.and_then(|ts| ts.get(i).copied()).unwrap_or([0.0, 0.0]);
                // glTF: Y-up, -Z forward → our: Y-up, +Z forward
                vertices.push(Vertex {
                    position: [p[0], p[1], -p[2]],
                    normal: [n[0], n[1], -n[2]],
                    tex_coord: t,
                });
            }

            if !has_normals {
                compute_flat_normals(&mut vertices[base..]);
            }

            loaded_any = true;
        }
    }

    if !loaded_any {
        return Err("No meshes found in glTF file".to_string());
    }

    let indices: Vec<u32> = (0..vertices.len() as u32).collect();
    println!("Loaded glTF: {} vertices, {} triangles", vertices.len(), indices.len() / 3);
    Ok(ModelObj { vertices, indices })
}

pub fn load_stl(path: &str) -> Result<ModelObj, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open STL: {}", e))?;

    let mut header = [0u8; 80];
    file.read_exact(&mut header).map_err(|e| format!("Failed to read STL header: {}", e))?;

    let mut count_buf = [0u8; 4];
    file.read_exact(&mut count_buf).map_err(|e| format!("Failed to read triangle count: {}", e))?;
    let triangle_count = u32::from_le_bytes(count_buf);

    let mut vertices = Vec::with_capacity(triangle_count as usize * 3);

    for i in 0..triangle_count {
        let mut tri = [0u8; 50];
        file.read_exact(&mut tri).map_err(|e| {
            format!("Failed to read STL triangle {}: {}", i, e)
        })?;

        for v in 0..3 {
            let base = 12 + v * 12;
            let x = f32::from_le_bytes(tri[base..base + 4].try_into().unwrap());
            let y = f32::from_le_bytes(tri[base + 4..base + 8].try_into().unwrap());
            let z = f32::from_le_bytes(tri[base + 8..base + 12].try_into().unwrap());
            vertices.push(Vertex {
                position: [x, y, z],
                normal: [0.0, 0.0, 0.0],
                tex_coord: [0.0, 0.0],
            });
        }
    }

    compute_flat_normals(&mut vertices);

    let indices: Vec<u32> = (0..vertices.len() as u32).collect();
    println!("Loaded STL: {} vertices, {} triangles", vertices.len(), indices.len() / 3);
    Ok(ModelObj { vertices, indices })
}

fn compute_flat_normals(vertices: &mut [Vertex]) {
    for chunk in vertices.chunks_mut(3) {
        if chunk.len() < 3 {
            continue;
        }
        let p0 = chunk[0].position;
        let p1 = chunk[1].position;
        let p2 = chunk[2].position;
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        let n = if len > 0.0 { [n[0] / len, n[1] / len, n[2] / len] } else { [0.0, 0.0, 1.0] };
        for v in chunk.iter_mut() {
            v.normal = n;
        }
    }
}

pub(crate) fn normalize_model(model: &mut ModelObj) {
    if model.vertices.is_empty() {
        return;
    }

    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for v in &model.vertices {
        for i in 0..3 {
            min[i] = min[i].min(v.position[i]);
            max[i] = max[i].max(v.position[i]);
        }
    }

    let center = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let extent = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let max_dim = extent[0].max(extent[1].max(extent[2]));

    if max_dim > MODEL_AUTO_SCALE_EXTENT {
        let scale = MODEL_AUTO_SCALE_EXTENT / max_dim;
        for v in &mut model.vertices {
            v.position = [
                (v.position[0] - center[0]) * scale,
                (v.position[1] - center[1]) * scale,
                (v.position[2] - center[2]) * scale,
            ];
        }
        println!("Auto-scaled model: original extent={:.3}, scale={:.3}", max_dim, scale);
    } else if max_dim > 0.0 {
        for v in &mut model.vertices {
            v.position = [
                v.position[0] - center[0],
                v.position[1] - center[1],
                v.position[2] - center[2],
            ];
        }
        println!("Model centered at origin, extent={:.3}", max_dim);
    }
}

pub(crate) fn default_model_obj() -> ModelObj {
    let vertices = default_vertices();
    let indices: Vec<u32> = (0..vertices.len() as u32).collect();
    ModelObj { vertices, indices }
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
    pub vertex_count: u32,
    pub color: [f32; 4],
    pub translation: [f32; 4],
    pub translation_base: [f32; 4],
    pub rotation: [f32; 4],
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
    pub texture_bind_group_nearest: BindGroup,
    pub texture_bind_group_linear: BindGroup,
}

impl ModelInstance {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model: ModelObj,
        device: &Device,
        queue: &Queue,
        translation: [f32; 4],
        translation_base: [f32; 4],
        rotation: [f32; 4],
        projection: [f32; 16],
        texture_path: &str,
        bind_group_layout: &BindGroupLayout,
        tex_bind_group_layout: &BindGroupLayout,
        color: [f32; 4],
    ) -> Self {
        let vertex_count = model.vertices.len() as u32;
        let index_count = model.indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&model.vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&model.indices),
            usage: BufferUsages::INDEX,
        });

        let init_uniforms = Uniforms {
            translation,
            rotation,
            view_proj: projection,
            use_texture: 1,
            _padding0: [0.0; 3],
            base_color: color,
            light_dir: LIGHT_DIR,
        };
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Model Uniform Buffer"),
            contents: bytemuck::cast_slice(&[init_uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Model Bind Group"),
            layout: bind_group_layout,
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

        let make_tex_bind_group = |sampler: &Sampler, label: &str| {
            device.create_bind_group(&BindGroupDescriptor {
                label: Some(label),
                layout: tex_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&loaded_texture.view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(sampler),
                    },
                ],
            })
        };

        let texture_bind_group_nearest =
            make_tex_bind_group(&loaded_texture.sampler_nearest, "Texture Bind Group (Nearest)");
        let texture_bind_group_linear =
            make_tex_bind_group(&loaded_texture.sampler_linear, "Texture Bind Group (Linear)");

        Self {
            vertex_count,
            color,
            translation,
            translation_base,
            rotation,
            vertex_buffer,
            index_buffer,
            index_count,
            uniform_buffer,
            bind_group,
            texture_bind_group_nearest,
            texture_bind_group_linear,
        }
    }

    pub fn update_transform(&self, queue: &Queue, view_proj: [f32; 16], use_texture: i32) {
        let uniforms = Uniforms {
            translation: self.translation,
            rotation: self.rotation,
            view_proj,
            use_texture,
            _padding0: [0.0; 3],
            base_color: self.color,
            light_dir: LIGHT_DIR,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_vertex(x: f32, y: f32, z: f32) -> Vertex {
        Vertex { position: [x, y, z], normal: [0.0, 0.0, 1.0], tex_coord: [0.0, 0.0] }
    }

    fn bbox_center(model: &ModelObj) -> ([f32; 3], [f32; 3]) {
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for v in &model.vertices {
            for i in 0..3 {
                min[i] = min[i].min(v.position[i]);
                max[i] = max[i].max(v.position[i]);
            }
        }
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        (center, max)
    }

    #[test]
    fn normalize_scales_down_large_model() {
        let mut model = ModelObj {
            vertices: vec![
                sample_vertex(0.0, 0.0, 0.0),
                sample_vertex(100.0, 0.0, 0.0),
                sample_vertex(100.0, 100.0, 0.0),
            ],
            indices: vec![0, 1, 2],
        };

        normalize_model(&mut model);

        let (center, max) = bbox_center(&model);
        let extent = 2.0 * max[0].max(max[1]).max(max[2]);
        assert!(extent <= MODEL_AUTO_SCALE_EXTENT + 0.001, "extent {} > {}", extent, MODEL_AUTO_SCALE_EXTENT);
        for (i, c) in center.iter().enumerate() {
            assert!(c.abs() < 0.001, "not centered on axis {}: {}", i, c);
        }
    }

    #[test]
    fn normalize_centers_small_model() {
        let mut model = ModelObj {
            vertices: vec![
                sample_vertex(10.0, 10.0, 10.0),
                sample_vertex(12.0, 10.0, 10.0),
                sample_vertex(10.0, 12.0, 10.0),
            ],
            indices: vec![0, 1, 2],
        };

        normalize_model(&mut model);

        let (center, _) = bbox_center(&model);
        for (i, c) in center.iter().enumerate() {
            assert!(c.abs() < 0.001, "not centered on axis {}: {}", i, c);
        }
    }

    #[test]
    fn stl_binary_parses_triangles() {
        let path = std::env::temp_dir().join("tmv_test_triangle.stl");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0u8; 80]);
        bytes.extend_from_slice(&1u32.to_le_bytes());
        let mut tri = Vec::new();
        tri.extend_from_slice(&[0.0f32; 3].into_iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>());
        for p in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            for c in p {
                tri.extend_from_slice(&c.to_le_bytes());
            }
        }
        tri.extend_from_slice(&[0u8; 2]);
        bytes.extend_from_slice(&tri);
        std::fs::write(&path, &bytes).unwrap();

        let model = load_stl(path.to_str().unwrap()).expect("STL should parse");
        assert_eq!(model.vertices.len(), 3);
        assert_eq!(model.indices.len(), 3);
        assert_eq!(model.vertices[0].position, [0.0, 0.0, 0.0]);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn obj_parses_triangles() {
        let path = std::env::temp_dir().join("tmv_test_triangle.obj");
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        std::fs::write(&path, obj).unwrap();

        let model = load_obj(path.to_str().unwrap()).expect("OBJ should parse");
        assert_eq!(model.vertices.len(), 3);
        assert_eq!(model.indices.len(), 3);

        let _ = std::fs::remove_file(&path);
    }
}
