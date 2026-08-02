use image::imageops;
use wgpu::*;

pub struct LoadedTexture {
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler_nearest: Sampler,
    pub sampler_linear: Sampler,
}

fn mip_level_count(width: u32, height: u32) -> u32 {
    (width.max(height) as f32).log2().floor() as u32 + 1
}

fn align_to(value: u32, alignment: u32) -> u32 {
    value.div_ceil(alignment) * alignment
}

fn make_sampler(device: &Device, filter: FilterMode) -> Sampler {
    device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        ..Default::default()
    })
}

impl LoadedTexture {
    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mips = mip_level_count(width, height);

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: mips,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mut level_image = rgba.clone();
        for mip in 0..mips {
            let lw = level_image.width();
            let lh = level_image.height();
            let bytes_per_row = align_to(lw * 4, COPY_BYTES_PER_ROW_ALIGNMENT);
            let mut padded = vec![0u8; (bytes_per_row * lh) as usize];
            for (i, pixel) in level_image.as_raw().chunks_exact(4).enumerate() {
                let row = i as u32 / lw;
                let col = i as u32 % lw;
                let dst = (row * bytes_per_row + col * 4) as usize;
                padded[dst..dst + 4].copy_from_slice(pixel);
            }

            queue.write_texture(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: mip,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                &padded,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(lh),
                },
                Extent3d {
                    width: lw,
                    height: lh,
                    depth_or_array_layers: 1,
                },
            );

            if lw > 1 || lh > 1 {
                let nw = (lw / 2).max(1);
                let nh = (lh / 2).max(1);
                level_image = imageops::resize(&level_image, nw, nh, imageops::FilterType::Triangle);
            }
        }

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler_nearest = make_sampler(device, FilterMode::Nearest);
        let sampler_linear = make_sampler(device, FilterMode::Linear);

        Ok(Self {
            texture,
            view,
            sampler_nearest,
            sampler_linear,
        })
    }

    pub fn from_path(
        device: &Device,
        queue: &Queue,
        path: &str,
        label: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(device, queue, &bytes, label)
    }
}
