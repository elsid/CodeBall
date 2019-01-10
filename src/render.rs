use serde::Serialize;
use crate::my_strategy::vec3::Vec3;

#[derive(Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color {r, g, b, a}
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Sphere {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub radius: f64,
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Sphere {
    pub const fn new(position: Vec3, radius: f64, color: Color) -> Self {
        Sphere {
            x: position.x(),
            y: position.y(),
            z: position.z(),
            radius,
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

type Text = String;

#[derive(Debug, Serialize, Clone)]
pub struct Line {
    pub x1: f64,
    pub y1: f64,
    pub z1: f64,
    pub x2: f64,
    pub y2: f64,
    pub z2: f64,
    pub width: f64,
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Line {
    pub const fn new(begin: Vec3, end: Vec3, width: f64, color: Color) -> Self {
        Line {
            x1: begin.x(),
            y1: begin.y(),
            z1: begin.z(),
            x2: end.x(),
            y2: end.y(),
            z2: end.z(),
            width,
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

#[derive(Serialize)]
pub enum Object {
    Sphere(Sphere),
    Text(Text),
    Line(Line),
}

impl Object {
    pub const fn sphere(position: Vec3, radius: f64, color: Color) -> Self {
        Object::Sphere(Sphere::new(position, radius, color))
    }

    pub const fn text(value: String) -> Self {
        Object::Text(value)
    }

    pub const fn line(begin: Vec3, end: Vec3, width: f64, color: Color) -> Self {
        Object::Line(Line::new(begin, end, width, color))
    }
}

pub struct Render {
    objects: Vec<Object>,
}

impl Render {
    pub fn new() -> Self {
        Render {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

impl Serialize for Render {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        serializer.collect_seq(self.objects.iter())
    }
}
