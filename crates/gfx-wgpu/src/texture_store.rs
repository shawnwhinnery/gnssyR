use gfx::driver::TextureHandle;
use std::collections::HashMap;

pub struct StoredTexture {
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

/// Persistent textures (not cleared each `begin_frame`).
pub struct TextureStore {
    textures: HashMap<TextureHandle, StoredTexture>,
    next_id: TextureHandle,
    sampler: wgpu::Sampler,
}

impl TextureStore {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bitmap_sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            textures: HashMap::new(),
            next_id: 1,
            sampler,
        }
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn get(&self, handle: TextureHandle) -> Option<&StoredTexture> {
        self.textures.get(&handle)
    }

    pub fn free(&mut self, handle: TextureHandle) {
        self.textures.remove(&handle);
    }

    /// Upload ARGB `u32` pixels (same packing as `SoftwareDriver`) as `Rgba8Unorm`.
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pixels: &[u32],
        width: u32,
        height: u32,
    ) -> TextureHandle {
        if width == 0 || height == 0 || pixels.len() != (width * height) as usize {
            return 0;
        }
        let handle = self.next_id;
        self.next_id = self.next_id.saturating_add(1).max(1);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bitmap_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let unpadded = width as usize * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let bytes_per_row = ((unpadded + align - 1) / align) * align;
        let mut staging = vec![0u8; bytes_per_row * height as usize];
        for y in 0..height as usize {
            for x in 0..width as usize {
                let p = pixels[y * width as usize + x];
                let r = ((p >> 16) & 0xFF) as u8;
                let g = ((p >> 8) & 0xFF) as u8;
                let b = (p & 0xFF) as u8;
                let a = ((p >> 24) & 0xFF) as u8;
                let base = y * bytes_per_row + x * 4;
                staging[base..base + 4].copy_from_slice(&[r, g, b, a]);
            }
        }

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &staging,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row as u32),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.textures
            .insert(handle, StoredTexture { texture, view });
        handle
    }
}
