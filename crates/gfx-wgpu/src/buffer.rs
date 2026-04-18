use gfx::driver::{MeshHandle, Vertex};
use std::mem;
use wgpu::util::DeviceExt;

/// GPU-side vertex + index buffers for a single uploaded mesh.
pub struct UploadedMesh {
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
}

/// Per-frame mesh pool.
///
/// All meshes are valid for the current frame only.  `clear` drops every
/// buffer and is called at the start of each `begin_frame`, satisfying the
/// spec contract that handles expire after the next `begin_frame`.
#[derive(Default)]
pub struct MeshPool {
    meshes: Vec<UploadedMesh>,
}

impl MeshPool {
    /// Upload `vertices` and `indices` to the GPU and return a [`MeshHandle`].
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> MeshHandle {
        // Vertex is #[repr(C)] with only f32 fields — safe to view as bytes.
        let vertex_bytes = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * mem::size_of::<Vertex>(),
            )
        };
        // u32 is always safely castable to bytes.
        let index_bytes = bytemuck::cast_slice::<u32, u8>(indices);

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_vertices"),
            contents: vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_indices"),
            contents: index_bytes,
            usage: wgpu::BufferUsages::INDEX,
        });

        let handle = self.meshes.len() as MeshHandle;
        self.meshes.push(UploadedMesh {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u32,
        });
        handle
    }

    pub fn get(&self, handle: MeshHandle) -> Option<&UploadedMesh> {
        self.meshes.get(handle as usize)
    }

    /// Drop all GPU buffers.  Call once per `begin_frame`.
    pub fn clear(&mut self) {
        self.meshes.clear();
    }
}
