use std::env;

use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use wgpu::*;

mod buffers;
mod camera;
mod constants;
mod egui_manager;
mod grid;
mod input;
mod loader;
mod models;
mod render;
mod setup;
mod texture;
mod ui_panels;

use buffers::*;
use camera::Camera;
use constants::*;
use egui_manager::EguiManager;
use glam::Mat4;
use grid::GridRenderer;
use input::handle_input;
use loader::load_model;
use models::*;
use render::*;
use setup::create_graphics;
use ui_panels::UiState;

fn surface_config(format: TextureFormat, width: u32, height: u32) -> SurfaceConfiguration {
    SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width,
        height,
        present_mode: PRESENT_MODE,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: FRAME_LATENCY,
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let model_path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| DEFAULT_MODEL_PATH.to_string());
    let texture_path = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| DEFAULT_TEXTURE_PATH.to_string());

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .expect("Failed to create window");

    let gfx = create_graphics(&window).await;
    let surface_format = gfx.surface_format;

    let window_size = window.inner_size();
    let mut buffers = init_buffers(window_size, &gfx.device);

    let translation = INITIAL_TRANSLATION;
    let rotation = INITIAL_ROTATION;

    let mut camera = Camera::new(
        glam::Vec3::new(0.0, 0.0, 4.5),
        CAMERA_INITIAL_DISTANCE,
        0.0,
        0.0,
    );

    let model_obj = load_model(&model_path).unwrap_or_else(|e| {
        eprintln!("Failed to load model '{}': {}", model_path, e);
        default_model_obj()
    });

    let mut models = vec![
        ModelInstance::new(
            model_obj,
            &gfx.device,
            &gfx.queue,
            translation,
            [0.0, 0.0, 0.0, 0.0],
            rotation,
            buffers.projection_matrix,
            &texture_path,
            &buffers.bind_group_layout,
            &buffers.texture_bind_group_layout,
            DEFAULT_BASE_COLOR,
        ),
    ];

    let shader_code = include_str!("shaders.wgsl");
    let shader_module = gfx.device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(shader_code.into()),
    });

    let grid = GridRenderer::new(&gfx.device, surface_format);

    let pipeline_layout = gfx.device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&buffers.bind_group_layout, &buffers.texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = gfx.device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        vertex: VertexState {
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &[
                    VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: VertexFormat::Float32x3,
                    },
                    VertexAttribute {
                        offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                        shader_location: 2,
                        format: VertexFormat::Float32x3,
                    },
                    VertexAttribute {
                        offset: (std::mem::size_of::<[f32; 3]>() * 2) as BufferAddress,
                        shader_location: 1,
                        format: VertexFormat::Float32x2,
                    },
                ],
            }],
            module: &shader_module,
            entry_point: "vs_main",
        },
        fragment: Some(FragmentState {
            targets: &[Some(ColorTargetState {
                format: surface_format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
            module: &shader_module,
            entry_point: "fs_main",
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        layout: Some(&pipeline_layout),
        depth_stencil: Some(buffers.depth_stencil),
        multisample: Default::default(),
        multiview: None,
    });

    gfx.surface.configure(
        &gfx.device,
        &surface_config(surface_format, window_size.width, window_size.height),
    );

    let mut input = WinitInputHelper::new();
    let mut egui_manager =
        EguiManager::new(&gfx.device, surface_format, None, DEFAULT_SAMPLE_COUNT, &window);
    let mut ui_state = UiState::new(model_path, texture_path);
    let win_id = window.id();

    let _ = event_loop.run(|event, event_loop_target| {
        if let Event::WindowEvent {
            event,
            window_id: ev_id,
        } = &event
        {
            if *ev_id == win_id {
                egui_manager.handle_input(&window, event);
            }
        }

        if input.update(&event) {
            handle_input(&mut input, &mut camera, &mut ui_state);
        }

        for model in &mut models {
            model.translation = [
                model.translation_base[0] + translation[0],
                model.translation_base[1] + translation[1],
                model.translation_base[2] + translation[2],
                model.translation_base[3] + translation[3],
            ];
        }

        let view = camera.build_view_matrix();
        let projection_mat = Mat4::from_cols_array(&buffers.projection_matrix);
        let view_proj = (projection_mat * view).to_cols_array();

        grid.update(&gfx.queue, view_proj);
        models[0].update_transform(&gfx.queue, view_proj, ui_state.use_texture as i32);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id: ev_id,
            } if ev_id == win_id => {
                event_loop_target.exit();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                ui_state.vertex_count = models[0].vertex_count;
                ui_state.triangle_count = models[0].index_count / 3;

                render(
                    &gfx.surface,
                    &gfx.device,
                    &gfx.queue,
                    &render_pipeline,
                    &models,
                    &buffers.depth_buffer.view,
                    &grid,
                    ui_state.show_grid,
                    ui_state.pixelated,
                    &mut egui_manager,
                    &window,
                    |ctx| ui_state.render(ctx),
                );
                ui_state.update_fps();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                window_id: ev_id,
            } if ev_id == win_id => {
                gfx.surface.configure(
                    &gfx.device,
                    &surface_config(surface_format, new_size.width, new_size.height),
                );
                buffers.depth_buffer.resize(&gfx.device, new_size);
                buffers.projection_matrix = create_perspective_matrix(
                    new_size.width as f32 / new_size.height as f32,
                    CAMERA_FOV,
                    CAMERA_NEAR,
                    CAMERA_FAR,
                );
            }
            _ => (),
        }
    });
}
