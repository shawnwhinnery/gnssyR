use crate::{driver::GraphicsDriver, path::Path, style::Style, transform::Transform};
use glam::Vec2;

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub origin: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(origin: Vec2, size: Vec2) -> Self {
        Self { origin, size }
    }
}

/// A [`Path`] with an associated [`Style`] and local [`Transform`].
pub struct Shape {
    pub path: Path,
    pub style: Style,
    pub transform: Transform,
}

impl Shape {
    pub fn new(path: Path, style: Style) -> Self {
        Self {
            path,
            style,
            transform: Transform::identity(),
        }
    }

    pub fn with_transform(mut self, t: Transform) -> Self {
        self.transform = t;
        self
    }
}

/// A named group of [`Node`]s sharing a common [`Transform`].
pub struct Group {
    pub children: Vec<Node>,
    pub transform: Transform,
}

impl Group {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            transform: Transform::identity(),
        }
    }

    pub fn with_transform(mut self, t: Transform) -> Self {
        self.transform = t;
        self
    }

    pub fn add(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Node {
    Shape(Shape),
    Group(Group),
}

impl From<Shape> for Node {
    fn from(s: Shape) -> Self {
        Node::Shape(s)
    }
}

impl From<Group> for Node {
    fn from(g: Group) -> Self {
        Node::Group(g)
    }
}

/// Root of the frame. Walk the tree, tessellate, and submit to a driver.
pub struct Scene {
    pub root: Group,
}

impl Scene {
    pub fn new() -> Self {
        Self { root: Group::new() }
    }

    pub fn add(&mut self, node: impl Into<Node>) {
        self.root.children.push(node.into());
    }

    /// Render the scene to `driver`.
    ///
    /// Calls `driver.begin_frame()`, walks the node tree, tessellates each
    /// [`Shape`], submits draw calls, then calls `driver.end_frame()`.
    /// An empty scene makes no driver calls at all.
    pub fn render(&self, driver: &mut dyn GraphicsDriver) {
        if self.root.children.is_empty() {
            return;
        }
        driver.begin_frame();
        render_group(&self.root, Transform::identity(), driver);
        driver.end_frame();
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

fn render_group(group: &Group, parent_transform: Transform, driver: &mut dyn GraphicsDriver) {
    let transform = parent_transform.then(group.transform);
    for node in &group.children {
        match node {
            Node::Shape(shape) => render_shape(shape, transform, driver),
            Node::Group(child) => render_group(child, transform, driver),
        }
    }
}

fn render_shape(shape: &Shape, parent_transform: Transform, driver: &mut dyn GraphicsDriver) {
    let _transform = parent_transform.then(shape.transform);
    let meshes = crate::path::tessellate::tessellate(&shape.path, &shape.style);
    for mesh in meshes {
        let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
        driver.draw_mesh(handle, _transform.to_mat3(), [1.0, 1.0, 1.0, 1.0]);
    }
}
