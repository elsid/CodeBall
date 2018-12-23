use std::collections::{HashMap, BTreeSet};
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

    pub fn set_color(&mut self, color: Color) {
        self.r = color.r;
        self.g = color.g;
        self.b = color.b;
        self.a = color.a;
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

    pub fn set_color(&mut self, color: Color) {
        self.r = color.r;
        self.g = color.g;
        self.b = color.b;
        self.a = color.a;
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Tag {
    Default,
    RobotId(i32),
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct SphereWrapper<'r> {
    Sphere: &'r Sphere,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct TextWrapper<'r> {
    Text: &'r Text,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct LineWrapper<'r> {
    Line: &'r Line,
}

struct Item {
    tag: Tag,
    object: Object,
}

pub struct Render {
    objects: HashMap<i32, Item>,
    include: BTreeSet<Tag>,
    exclude: BTreeSet<Tag>,
    next_id: i32,
}

impl Render {
    pub fn new() -> Self {
        Render {
            objects: HashMap::new(),
            include: BTreeSet::new(),
            exclude: BTreeSet::new(),
            next_id: 0,
        }
    }

    pub fn add(&mut self, object: Object) -> i32 {
        let id = self.next_id;
        self.objects.insert(id, Item {tag: Tag::Default, object});
        self.next_id += 1;
        id
    }

    pub fn add_with_tag(&mut self, tag: Tag, object: Object) -> i32 {
        let id = self.next_id;
        self.objects.insert(id, Item {tag, object});
        self.next_id += 1;
        id
    }

    pub fn get(&mut self, id: i32) -> Option<&Object> {
        self.objects.get(&id).map(|v| &v.object)
    }

    pub fn get_mut(&mut self, id: i32) -> Option<&mut Object> {
        self.objects.get_mut(&id).map(|v| &mut v.object)
    }

    pub fn get_sphere(&mut self, id: i32) -> Option<&Sphere> {
        if let Some(Object::Sphere(ref v)) = self.get(id) {
            Some(&v)
        } else {
            None
        }
    }

    pub fn get_sphere_mut(&mut self, id: i32) -> Option<&mut Sphere> {
        if let Some(Object::Sphere(ref mut v)) = self.get_mut(id) {
            Some(v)
        } else {
            None
        }
    }

    pub fn include_tag(&mut self, tag: Tag) {
        self.exclude.remove(&tag);
        self.include.insert(tag);
    }

    pub fn exclude_tag(&mut self, tag: Tag) {
        self.include.remove(&tag);
        self.exclude.insert(tag);
    }

    pub fn ignore_tag(&mut self, tag: Tag) {
        self.include.remove(&tag);
        self.exclude.remove(&tag);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

impl Serialize for Render {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        serializer.collect_seq(
            self.objects.iter()
                .filter(|(_, v)| {
                    self.include.contains(&v.tag)
                        || (self.include.is_empty() && !self.exclude.contains(&v.tag))
                })
                .map(|(_, v)| &v.object)
        )
    }
}
