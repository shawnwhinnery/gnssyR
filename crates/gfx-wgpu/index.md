# gfx-wgpu

GPU-backed `GraphicsDriver` implementation using `wgpu`. Production driver targeting Vulkan on Linux/Steam Deck, Metal on macOS, and DX12 on Windows.

---

## Construction

- Created from a window handle (`raw-window-handle`)
- Initialises the wgpu device, queue, and swapchain surface
- Panics if no compatible GPU adapter is found

---

## Behaviour

- Satisfies the full `GraphicsDriver` trait contract (defined in `gfx`)
- Mesh data is uploaded to GPU vertex/index buffers on `upload_mesh`
- Buffers allocated in a frame are recycled at the next `begin_frame`
- `present` submits the command queue and presents the swapchain image

---

## Platform Requirements

- Requires a GPU with Vulkan 1.1 / Metal 2 / DX12 support
- Not suitable for headless CI environments without GPU passthrough
