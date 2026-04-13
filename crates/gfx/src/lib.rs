pub mod color;
pub mod driver;
pub mod path;
pub mod scene;
pub mod shape;
pub mod style;
pub mod transform;

pub use color::Color;
pub use driver::{GraphicsDriver, Mesh, MeshHandle, Vertex};
pub use glam::Vec2;
pub use path::Path;
pub use scene::{Group, Node, Scene, Shape};
pub use style::{Fill, LineCap, LineJoin, Stroke, Style};
pub use transform::Transform;

/// Tessellate a path with the given style into GPU-ready triangle meshes.
///
/// Returns one mesh per enabled style component (fill and/or stroke).
/// The vertices are in the same coordinate space as the path.
pub fn tessellate(path: &path::Path, style: &style::Style) -> Vec<driver::Mesh> {
    path::tessellate::tessellate(path, style)
}
