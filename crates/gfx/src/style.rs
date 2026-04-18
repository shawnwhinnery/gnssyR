use crate::color::Color;
use glam::Vec2;

#[derive(Debug, Clone)]
pub struct ColorStop {
    pub offset: f32, // 0.0 – 1.0
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum Fill {
    Solid(Color),
    LinearGradient {
        start: Vec2,
        end: Vec2,
        stops: Vec<ColorStop>,
    },
    RadialGradient {
        center: Vec2,
        radius: f32,
        stops: Vec<ColorStop>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
    pub cap: LineCap,
    pub join: LineJoin,
}

impl Stroke {
    pub fn solid(color: Color, width: f32) -> Self {
        Self {
            color,
            width,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

impl Style {
    pub fn filled(color: Color) -> Self {
        Self {
            fill: Some(Fill::Solid(color)),
            stroke: None,
        }
    }

    pub fn stroked(stroke: Stroke) -> Self {
        Self {
            fill: None,
            stroke: Some(stroke),
        }
    }
}
