use egui::{Context, Visuals};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use winit::window::Window;

pub struct EguiManager {
    pub context: Context,
    state: State,
    renderer: Renderer,
}

impl EguiManager {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> Self {
        let egui_context = Context::default();
        let id = egui_context.viewport_id();

        let mut style = (*egui_context.style()).clone();
        style.visuals = Visuals::dark();
        style.visuals.panel_fill = egui::Color32::from_rgb(38, 38, 42);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(48, 48, 54);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(58, 58, 64);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(68, 68, 76);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(78, 78, 86);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(61, 110, 245);
        style.visuals.hyperlink_color = egui::Color32::from_rgb(61, 110, 245);
        style.visuals.window_rounding = egui::Rounding::same(0.0);
        style.visuals.window_shadow = egui::epaint::Shadow::NONE;
        style.spacing.item_spacing = egui::Vec2::new(8.0, 4.0);
        style.spacing.button_padding = egui::Vec2::new(6.0, 2.0);
        egui_context.set_style(style);

        let state = State::new(egui_context.clone(), id, window, None, None);
        let renderer = Renderer::new(device, output_color_format, output_depth_format, msaa_samples);

        Self {
            context: egui_context,
            state,
            renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &winit::event::WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn draw<F>(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
        run_ui: F,
    ) where
        F: FnOnce(&Context),
    {
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, |_ui| {
            run_ui(&self.context);
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer.render(&mut render_pass, &tris, &screen_descriptor);
        drop(render_pass);

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
