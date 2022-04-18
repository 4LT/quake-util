extern crate alloc;

use crate::qmap::repr::*;

#[cfg(feature = "std")]
use std::ffi::CString;

#[cfg(feature = "alloc_fills")]
use cstr_core::CString;

const GOOD_AXES: [Vec3; 2] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

const BAD_AXES: [Vec3; 2] = [[f64::INFINITY, 0.0, 0.0], [0.0, 0.0, 0.0]];

const GOOD_VEC2: Vec2 = [1.0, 1.0];

const BAD_VEC2: Vec2 = [-f64::INFINITY, 0.0];

const GOOD_HALF_SPACE: [Point; 3] =
    [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, 1.0, 0.0]];

const BAD_HALF_SPACE: [Point; 3] =
    [[f64::NAN, -1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, 1.0, 0.0]];

const GOOD_ALIGNMENT: Alignment = Alignment::Valve220(
    BaseAlignment {
        offset: GOOD_VEC2,
        rotation: 0.0,
        scale: GOOD_VEC2,
    },
    GOOD_AXES,
);

fn panic_expected_error() {
    panic!("Expected error");
}

fn simple_edict() -> Edict {
    let mut edict = Edict::new();
    edict.insert(
        CString::new("classname").unwrap(),
        CString::new("worldspawn").unwrap(),
    );
    edict
}

fn bad_edict_key() -> Edict {
    let mut edict = Edict::new();
    edict.insert(CString::new("\n").unwrap(), CString::new("oops").unwrap());
    edict
}

fn simple_surface() -> Surface {
    Surface {
        half_space: GOOD_HALF_SPACE,
        texture: CString::new("{FENCE").unwrap(),
        alignment: GOOD_ALIGNMENT,
    }
}

fn bad_surface_texture() -> Surface {
    Surface {
        half_space: GOOD_HALF_SPACE,
        texture: CString::new("\"").unwrap(),
        alignment: GOOD_ALIGNMENT,
    }
}

fn simple_brush() -> Brush {
    vec![
        simple_surface(),
        simple_surface(),
        simple_surface(),
        simple_surface(),
    ]
}

fn simple_brush_entity() -> Entity {
    Entity::Brush(simple_edict(), vec![simple_brush()])
}

fn simple_point_entity() -> Entity {
    Entity::Point(simple_edict())
}

fn bad_entity_edict() -> Entity {
    Entity::Brush(bad_edict_key(), vec![simple_brush()])
}

fn simple_map() -> QuakeMap {
    let mut qmap = QuakeMap::new();
    qmap.entities.push(simple_brush_entity());
    qmap.entities.push(simple_point_entity());
    qmap
}

fn bad_map_edict() -> QuakeMap {
    let mut qmap = QuakeMap::new();
    let ent = bad_entity_edict();
    qmap.entities.push(ent);
    qmap
}

#[test]
fn mutable_edict() {
    let mut ent = Entity::Point(Edict::new());
    let key = CString::new("skin").unwrap();
    let value = CString::new("value").unwrap();

    ent.edict_mut().insert(key.clone(), value.clone());

    assert_eq!(ent.edict().get(&key), Some(&value));
}

#[test]
fn mutable_base_alignment() {
    let mut alignment = GOOD_ALIGNMENT;

    alignment.base_mut().rotation = 12.0;

    assert_eq!(alignment.base().rotation, 12.0);
}

#[cfg(feature = "std")]
mod write {
    use super::*;
    use std::io::sink;
    use std::string::String;
    use std::vec::Vec;

    fn expect_err_containing<T>(res: std::io::Result<T>, text: &str) {
        if let Err(e) = res {
            let s = format!("{:?}", e);
            assert!(s.contains(text), "Expected {:?} to contain '{}'", e, text);
        } else {
            panic_expected_error();
        }
    }

    // Successes

    #[test]
    fn write_empty_map() {
        let map = QuakeMap::new();
        let mut dest: Vec<u8> = vec![];
        assert!(map.write_to(&mut dest).is_ok());
        assert_eq!(&dest[..], b"");
    }

    #[test]
    fn write_simple_map() {
        let mut dest: Vec<u8> = vec![];
        assert!(simple_map().write_to(&mut dest).is_ok());
        assert!(String::from_utf8(dest).unwrap().contains("worldspawn"));
    }

    // Failure

    #[test]
    fn write_bad_map() {
        let res = bad_map_edict().write_to(&mut sink());
        if res.is_ok() {
            panic_expected_error();
        }
    }

    #[test]
    fn write_bad_entity() {
        let res = bad_entity_edict().write_to(&mut sink());
        expect_err_containing(res, "\\n");
    }

    #[test]
    fn write_bad_surface_texture() {
        let res = bad_surface_texture().write_to(&mut sink());
        expect_err_containing(res, "\\\"");
    }

    #[test]
    fn write_bad_surface_half_space() {
        let surf = Surface {
            half_space: BAD_HALF_SPACE,
            texture: CString::new("butts").unwrap(),
            alignment: GOOD_ALIGNMENT,
        };

        let res = surf.write_to(&mut sink());
        expect_err_containing(res, "finite");
    }

    #[test]
    fn write_bad_alignment_offset() {
        let aln = Alignment::Standard(BaseAlignment {
            offset: BAD_VEC2,
            rotation: 0.0,
            scale: GOOD_VEC2,
        });

        let res = aln.write_to(&mut sink());
        expect_err_containing(res, "finite");
    }

    #[test]
    fn write_bad_alignment_rotation() {
        let aln = Alignment::Standard(BaseAlignment {
            offset: GOOD_VEC2,
            rotation: f64::NAN,
            scale: GOOD_VEC2,
        });

        let res = aln.write_to(&mut sink());
        expect_err_containing(res, "finite");
    }

    #[test]
    fn write_bad_alignment_scale() {
        let aln = Alignment::Standard(BaseAlignment {
            offset: GOOD_VEC2,
            rotation: 0.7,
            scale: BAD_VEC2,
        });

        let res = aln.write_to(&mut sink());
        expect_err_containing(res, "finite");
    }

    #[test]
    fn write_bad_alignment_axes() {
        let aln = Alignment::Valve220(
            BaseAlignment {
                offset: GOOD_VEC2,
                rotation: -0.37,
                scale: GOOD_VEC2,
            },
            BAD_AXES,
        );

        let res = aln.write_to(&mut sink());
        expect_err_containing(res, "finite");
    }
}
