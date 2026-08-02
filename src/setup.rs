use wgpu::*;
use winit::window::Window;

pub struct GraphicsSetup<'a> {
    pub surface: Surface<'a>,
    pub device: Device,
    pub queue: Queue,
    pub surface_format: TextureFormat,
}

pub async fn create_graphics<'a>(window: &'a Window) -> GraphicsSetup<'a> {
    let instance = Instance::new(InstanceDescriptor::default());
    let surface = instance
        .create_surface(window)
        .expect("Failed to create window surface");

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .expect("No compatible GPU adapter found");

    println!("{}", adapter.get_info().name);

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                required_features: Features::empty(),
                required_limits: Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .expect("Failed to request GPU device");

    let caps = surface.get_capabilities(&adapter);
    let surface_format = caps.formats[0];

    GraphicsSetup {
        surface,
        device,
        queue,
        surface_format,
    }
}
