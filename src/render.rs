use wgpu::*;

use crate::egui_manager::EguiManager;
use crate::grid::GridRenderer;
use crate::ModelInstance;

#[allow(clippy::too_many_arguments)]
pub fn render(
    surface: &Surface,
    device: &Device,
    queue: &Queue,
    render_pipeline: &RenderPipeline,
    models: &[ModelInstance],
    depth_view: &TextureView,
    grid: &GridRenderer,
    show_grid: bool,
    pixelated: bool,
    egui_manager: &mut EguiManager,
    window: &winit::window::Window,
    run_ui: impl FnOnce(&egui::Context),
) {
    let frame = match surface.get_current_texture() {
        Ok(frame) => frame,
        Err(_) => return,
    };

    let view = frame
        .texture
        .create_view(&TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.02,
                        g: 0.02,
                        b: 0.02,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        if show_grid {
            grid.render(&mut render_pass);
        }

        render_pass.set_pipeline(render_pipeline);

        for model in models {
            let tex_bind_group = if pixelated {
                &model.texture_bind_group_nearest
            } else {
                &model.texture_bind_group_linear
            };
            render_pass.set_bind_group(0, &model.bind_group, &[]);
            render_pass.set_bind_group(1, tex_bind_group, &[]);
            render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
            render_pass.set_index_buffer(model.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..model.index_count, 0, 0..1);
        }
    }

    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [frame.texture.width(), frame.texture.height()],
        pixels_per_point: window.scale_factor() as f32,
    };

    egui_manager.draw(
        device,
        queue,
        &mut encoder,
        window,
        &view,
        screen_descriptor,
        run_ui,
    );

    queue.submit(std::iter::once(encoder.finish()));
    frame.present();
}
