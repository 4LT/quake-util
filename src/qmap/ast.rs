use std::io;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

pub type Point = [f64; 3];
pub type Vec3 = [f64; 3];
pub type Vec2 = [f64; 2];

pub trait AstElement<W: io::Write> {
    fn write_to(&self, writer: &mut W) -> io::Result<()>;
}

pub struct QuakeMap {
    pub entities: Vec<Entity>
}

impl<W: io::Write> AstElement<W> for QuakeMap {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        for ent in &self.entities {
            ent.write_to(writer)?;
        }
        Ok(())
    }
}

pub enum Entity {
    Brush(Edict, Vec<Brush>),
    Point(Edict)
}

impl<W: io::Write> AstElement<W> for Entity {
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

pub type Edict = HashMap<Vec<u8>, Vec<u8>>;

impl<W: io::Write> AstElement<W> for Edict {
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

pub type Brush = Vec<Surface>;

impl<W: io::Write> AstElement<W> for Brush {
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

pub struct Surface {
    pub half_space: HalfSpace,
    pub texture: Vec<u8>,
    pub alignment: Alignment
}

impl<W: io::Write> AstElement<W> for Surface {
    fn write_to(&self, writer: &mut W) -> io::Result<()> {
        self.half_space.write_to(writer)?;
        writer.write_all(b" ")?;
        writer.write_all(&self.texture)?;
        writer.write_all(b" ")?;
        self.alignment.write_to(writer)?;
        Ok(())
    }
}

pub struct HalfSpace(pub Point, pub Point, pub Point);

impl HalfSpace {
    fn iter(&self) -> impl Iterator<Item=Point> + '_ {
        (0usize..3usize).map(move | i | self[i])
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

impl<W: io::Write> AstElement<W> for HalfSpace {
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

pub enum Alignment {
    Standard(BaseAlignment),
    Valve220 {
        base: BaseAlignment,
        u: Vec3,
        v: Vec3
    }
}

impl<W: io::Write> AstElement<W> for Alignment {
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

pub struct BaseAlignment {
    pub offset: Vec2,
    pub rotation: f64,
    pub scale: Vec2
}
