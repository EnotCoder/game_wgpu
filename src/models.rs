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
        break;
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
        model: ModelObj,
        device: &Device,
        queue: &Queue,
        translation: [f32; 4],
        translation_base: [f32; 4],
        rotation: [f32; 4],
        projection: [f32; 16],
        texture_path: &str,
    ) -> Self {
        let (vertices, indices_u32) = (model.vertices, model.indices);

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
            view_proj: projection,
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

    pub fn update_transform(&self, queue: &Queue, view_proj: [f32; 16], use_texture: i32) {
        let uniforms = Uniforms {
            translation: self.translation,
            rotation: self.rotation,
            view_proj,
            use_texture,
            _padding0: [0.0; 3],
            light_dir: LIGHT_DIR,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}
