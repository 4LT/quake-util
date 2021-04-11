use std::io;
use std::iter;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

pub type Point = [f64; 3];
pub type Vec3 = [f64; 3];
pub type Vec2 = [f64; 2];

pub type BoxedValidateIterator<'a> = Box<dyn Iterator<Item=String> + 'a>;

pub trait Writes<W: io::Write> {
    fn write_to(&self, writer: &mut W) -> io::Result<()>;
}

pub trait Validate<'a> {
    fn validate(&'a self) -> BoxedValidateIterator<'a>;
}

pub trait AstElement<'a, W: io::Write>: Writes<W> + Validate<'a> {}

pub struct QuakeMap {
    pub entities: Vec<Entity>
}

impl<W: io::Write> Writes<W> for QuakeMap {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for ent in &self.entities {
            ent.write_to(writer)?;
        }
        Ok(())
    }
}

impl<'a> Validate<'a> for QuakeMap {
    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        let worldspawn_classname_msg = String::from(
            "Entity 0: Must have classname of `worldspawn`");

        let worldspawn_brush_msg = String::from(
            "Entity 0: Must be a brush entity");

        let validate_worldspawn_edict =
            |edict: &Edict| {
                if let Some(classname) = edict.get(
                    &b"classname"[..]
                ) {
                    if classname == b"worldspawn" {
                        None.into_iter()
                    } else {
                        Some(worldspawn_classname_msg).into_iter()
                    }
                } else {
                    Some(worldspawn_classname_msg).into_iter()
                }
            };

        if self.entities.is_empty() {
            return Box::new(iter::once(String::from("Zero entities in map")))
        } 

        let validate_worldspawn: BoxedValidateIterator<'a>
            = match &self.entities[0] {
                Entity::Point(edict) => 
                    Box::new(validate_worldspawn_edict(&edict).chain(
                        iter::once(worldspawn_brush_msg))),
                Entity::Brush(edict, _) => 
                    Box::new(validate_worldspawn_edict(&edict)),
            };

        Box::new(validate_worldspawn.chain(self.entities
            .iter()
            .enumerate()
            .map(|(idx, ent)| {
                let prepend_index = move |msg| format!(
                    "Entity {}: {}",
                    idx, msg);
                ent.validate().map(prepend_index)
            })
            .flatten()))
    }
}

pub enum Entity {
    Brush(Edict, Vec<Brush>),
    Point(Edict)
}

impl<W: io::Write> Writes<W> for Entity {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"{\r\n")?;
        match self {
            Entity::Brush(edict, brushes) => {
                edict.write_to(writer)?;
                for brush in brushes {
                    brush.write_to(writer)?;
                }
            },
            Entity::Point(edict) => {
                edict.write_to(writer)?;
            }
        }
        writer.write_all(b"}\r\n")?;
        Ok(())
    }
}

impl<'a> Validate<'a> for Entity {
    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        match self {
            Entity::Point(edict) => edict.validate(),
            Entity::Brush(edict, brushes) => {
                let validate_brushes = brushes.iter()
                    .enumerate()
                    .map(|(idx, brush)| {
                        let prepend_brush = move |msg| format!(
                            "Brush {}: {}",
                            idx, msg);
                        brush.validate().map(prepend_brush)
                    }).flatten();

                if brushes.is_empty() {
                    Box::new(edict.validate().chain(
                        iter::once(
                            String::from("Brush entity with 0 brushes"))))
                } else {
                    Box::new(edict.validate().chain(validate_brushes))
                }
            }
        }
    }
}

pub type Edict = HashMap<Vec<u8>, Vec<u8>>;

impl<W: io::Write> Writes<W> for Edict {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for (key, value) in self {
            writer.write_all(b"\"")?;
            writer.write_all(key)?;
            writer.write_all(b"\" \"")?;
            writer.write_all(value)?;
            writer.write_all(b"\"\r\n")?;
        }
        Ok(())
    }
}

impl<'a> Validate<'a> for Edict {
    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        let validate_classname
            = match self.get(&b"classname"[..]) {
                None => Some(String::from(
                        "Missing classname")).into_iter(),
                Some(v) if v == b"" => Some(String::from(
                        "Empty classname")).into_iter(),
                _ => None.into_iter()
            };

        let validate_keyvalues = self.iter().map(
            |(k, v)| {
                let check_char = |&ch| ch == b'\0' || ch == b'"';
                let k_string = String::from_utf8_lossy(k);

                let validate_k
                    = if k.iter().any(check_char) {
                        Some(format!(
                                "Key `{}` contains illegal characters",
                                k_string)).into_iter()
                    } else {
                        None.into_iter()
                    };

                let validate_v
                    = if v.iter().any(check_char) {
                        let v_string = String::from_utf8_lossy(v);
                        Some(format!(
                                "Key `{}` has value `{}` \
                                with illegal characters",
                                k_string, v_string)).into_iter()
                    } else {
                        None.into_iter()
                    };

                validate_k.chain(validate_v)
            }).flatten();

        Box::new(validate_classname.chain(validate_keyvalues))
    }
}

pub type Brush = Vec<Surface>;

impl<W: io::Write> Writes<W> for Brush {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"{\r\n")?;

        for surf in self {
            surf.write_to(writer)?;
            writer.write_all(b"\r\n")?;
        }

        writer.write_all(b"}\r\n")?;
        Ok(())
    }
}

impl<'a> Validate<'a> for Brush {

    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        let validate_surface_count = if self.len() < 4 {
            Some(String::from("Surface count < 4")).into_iter()
        } else {
            None.into_iter()
        };

        let validate_surfaces = self.iter()
            .enumerate()
            .map(|(idx, surf)| {
                let prepend_surf = move |msg| format!(
                    "Surface {}: {}",
                    idx, msg);
                surf.validate().map(prepend_surf)
            }).flatten();

        Box::new(validate_surface_count.chain(validate_surfaces))
    }
}

pub struct Surface {
    pub half_space: HalfSpace,
    pub texture: Vec<u8>,
    pub alignment: Alignment
}

impl<W: io::Write> Writes<W> for Surface {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        self.half_space.write_to(writer)?;
        writer.write_all(b" ")?;
        writer.write_all(&self.texture)?;
        writer.write_all(b" ")?;
        self.alignment.write_to(writer)?;
        Ok(())
    }
}

impl<'a> Validate<'a> for Surface {

    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        let check_char = |&ch: &u8| ch == b'\0' || ch.is_ascii_whitespace();

        let validate_texture = if self.texture.iter().any(check_char) {
            Some(format!(
                    "Texture `{}` has illegal characters",
                    String::from_utf8_lossy(&self.texture))).into_iter()
        } else {
            None.into_iter()
        };

        Box::new(self.half_space.validate()
                 .chain(validate_texture)
                 .chain(self.alignment.validate()))
    }
}

pub struct HalfSpace(pub Point, pub Point, pub Point);

impl HalfSpace {
    fn iter(&self) -> impl Iterator<Item=&Point> + '_ {
        (0usize..3usize).map(move | i | &self[i])
    }
}

impl Index<usize> for HalfSpace {
    type Output = Point;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Out of bounds")
        }
    }
}

impl IndexMut<usize> for HalfSpace {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Out of bounds")
        }
    }
}

impl<W: io::Write> Writes<W> for HalfSpace {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for (index, pt) in self.iter().enumerate() {
            writer.write_all(b"( ")?;

            for element in pt.iter() {
                write!(writer, "{} ", element)?;
            }

            writer.write_all(b")")?;

            if index != 2 {
                writer.write_all(b" ")?;
            }
        }
        Ok(())
    }
}


impl<'a> Validate<'a> for HalfSpace {
    #[allow(clippy::float_cmp)]
    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        let validate_coords = self.iter()
            .map(|pt| pt.iter())
            .flatten()
            .map(|float| float.validate())
            .flatten();

        let validate_triangle =
            if self.0 == self.1
                || self.0 == self.2
                || self.1 == self.2
            {
                Some(String::from("Degenerate triangle")).into_iter()
            } else {
                None.into_iter()
            };

        Box::new(validate_coords.chain(validate_triangle))
    }
}

pub enum Alignment {
    Standard(BaseAlignment),
    Valve220 {
        base: BaseAlignment,
        u: Vec3,
        v: Vec3
    }
}

impl<W: io::Write> Writes<W> for Alignment {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Alignment::Standard(base) => {
                write!(writer,
                       "{} {} {} {} {}",
                       base.offset[0], base.offset[1],
                       base.rotation,
                       base.scale[0], base.scale[1])?;
            },
            Alignment::Valve220 { base, u, v } => {
                write!(writer,
                       "[ {} {} {} {} ] [ {} {} {} {} ] {} {} {}",
                       u[0], u[1], u[2], base.offset[0],
                       v[0], v[1], v[2], base.offset[1],
                       base.rotation,
                       base.scale[0], base.scale[1])?;
            }
        }
        Ok(())
    }
}

impl<'a> Validate<'a> for Alignment {
    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        let validate_tex_axis
            = |vec: &'a Vec3| vec.iter()
                .map(|float| float.validate())
                .flatten()
                .chain(if vec[0] == 0.0 && vec[1] == 0.0 && vec[2] == 0.0 {
                    Some(format!(
                        "Texture axis `{} {} {}` is directionless",
                        vec[0], vec[1], vec[2])).into_iter()
                } else {
                    None.into_iter()
                });

        match self {
            Alignment::Standard(base) => Box::new(base.validate()),
            Alignment::Valve220 { base, u, v } => Box::new(
                base.validate()
                    .chain(validate_tex_axis(u))
                    .chain(validate_tex_axis(v)))
        }
    }
}

pub struct BaseAlignment {
    pub offset: Vec2,
    pub rotation: f64,
    pub scale: Vec2
}

impl<'a> Validate<'a> for BaseAlignment {
    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        Box::new(self.offset.iter()
                 .map(|float| float.validate())
                 .flatten()
                 .chain(self.rotation.validate())
                 .chain(self.scale.iter()
                        .map(|float| float.validate())
                        .flatten()))
    }
}

impl<'a> Validate<'a> for f64 {
    fn validate(&'a self) -> BoxedValidateIterator<'a> {
        Box::new(if self.is_finite() {
            None.into_iter()
        } else {
            Some(format!("Non-finite number `{}`", self)).into_iter()
        })
    }
}
