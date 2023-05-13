use std::{collections::HashMap, fmt, path::Path, sync::Arc};

use raybow::{
    geometry::{bvh::Bvh, Object, Sphere},
    material::{Dialectric, DiffuseLight, Lambertian, Material, Metal},
    vector::Vector,
    Camera, Color,
};
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};

#[derive(Default, Clone, Copy, Deserialize)]
struct Point(f32, f32, f32);

impl From<Point> for Vector {
    fn from(point: Point) -> Self {
        Self::from_xyz(point.0, point.1, point.2)
    }
}

fn default_up_vector() -> (f32, f32, f32) {
    (0.0, 1.0, 0.0)
}

#[derive(Deserialize)]
struct CameraDesc {
    position: Point,
    #[serde(default)]
    lookat: Point,
    #[serde(default = "default_up_vector")]
    up: (f32, f32, f32),
    vfov: f32,
    focus_distance: Option<f32>,
    aperture: f32,
}

struct ColorVisitor;

impl ColorVisitor {
    fn parse_channel<E: serde::de::Error>(&self, string: &str) -> Result<u8, E> {
        match u8::from_str_radix(string, 16) {
            Ok(val) => Ok(val),
            Err(_) => Err(serde::de::Error::invalid_value(
                Unexpected::Str(string),
                self,
            )),
        }
    }
}

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a color in #RRGGBB format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if !v.starts_with('#') || !v.is_ascii() {
            return Err(serde::de::Error::invalid_value(Unexpected::Str(v), &self));
        }

        let r = self.parse_channel(&v[1..3])?;
        let g = self.parse_channel(&v[3..5])?;
        let b = self.parse_channel(&v[5..7])?;

        Ok(Color::from_rgb_bytes(r, g, b))
    }
}

fn deserialize_color<'de, D>(d: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_str(ColorVisitor)
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum MaterialDesc {
    Lambertian {
        #[serde(deserialize_with = "deserialize_color")]
        albedo: Color,
    },
    Metal {
        #[serde(deserialize_with = "deserialize_color")]
        albedo: Color,
        #[serde(default)]
        fuzz: f32,
    },
    Dialectric {
        refraction_index: f32,
    },
    DiffuseLight {
        #[serde(deserialize_with = "deserialize_color")]
        emit: Color,
    },
}

impl From<&MaterialDesc> for Arc<dyn Material> {
    fn from(desc: &MaterialDesc) -> Self {
        match desc {
            MaterialDesc::Lambertian { albedo } => Arc::new(Lambertian { albedo: *albedo }),
            MaterialDesc::Metal { albedo, fuzz } => Arc::new(Metal {
                albedo: *albedo,
                fuzz: *fuzz,
            }),
            MaterialDesc::Dialectric { refraction_index } => Arc::new(Dialectric {
                index: *refraction_index,
            }),
            MaterialDesc::DiffuseLight { emit } => Arc::new(DiffuseLight { emit: *emit }),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ObjectDesc {
    Sphere {
        center: Point,
        radius: f32,
        material: String,
    },
    Mesh {
        vertices: Vec<f32>,
        indices: Vec<u32>,
        material: String,
    },
}

#[derive(Deserialize)]
pub struct Scene {
    camera: CameraDesc,
    materials: HashMap<String, MaterialDesc>,
    objects: Vec<ObjectDesc>,
    #[serde(deserialize_with = "deserialize_color", default)]
    pub background: Color,
}

impl Scene {
    pub fn from_file(file: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(ron::from_str(&std::fs::read_to_string(file)?)?)
    }

    pub fn construct_camera(&self, aspect_ratio: f32) -> Camera {
        let desc = &self.camera;
        let position = desc.position.into();
        let lookat = desc.lookat.into();

        Camera::new(
            position,
            lookat,
            Vector::from_xyz(desc.up.0, desc.up.1, desc.up.2),
            desc.vfov,
            aspect_ratio,
            desc.aperture,
            desc.focus_distance
                .unwrap_or_else(|| (lookat - position).length()),
        )
    }

    pub fn construct_world(&self) -> Vec<Arc<dyn Object>> {
        let mut objects = Vec::<Arc<dyn Object>>::new();
        let materials: HashMap<String, Arc<dyn Material>> = self
            .materials
            .iter()
            .map(|(name, desc)| (name.clone(), desc.into()))
            .collect();

        for object_desc in &self.objects {
            match object_desc {
                ObjectDesc::Sphere {
                    center,
                    radius,
                    material,
                } => {
                    let material = materials.get(material).expect("undefined material");
                    let sphere = Sphere::new((*center).into(), *radius, Arc::clone(material));
                    objects.push(Arc::new(sphere));
                }
                ObjectDesc::Mesh {
                    vertices,
                    indices,
                    material,
                } => {
                    let material = materials.get(material).expect("undefined material");
                    let mesh = raybow::geometry::TriangleMesh::new(
                        vertices.clone().into_boxed_slice(),
                        indices.clone().into_boxed_slice(),
                        Arc::clone(material),
                    );
                    objects.push(Arc::new(Bvh::new(mesh)));
                }
            }
        }

        objects
    }
}
