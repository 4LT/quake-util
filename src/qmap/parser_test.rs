use crate::qmap;
use qmap::parser::parse;
use qmap::{Alignment, Entity};
use std::ffi::CString;
use std::io;

struct ErroringReader {}

impl ErroringReader {
    fn new() -> Self {
        Self {}
    }
}

impl io::Read for ErroringReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, "Generic test error"))
    }
}

fn panic_expected_brush() {
    panic!("Expected brush entity, found point entity");
}

fn panic_expected_point() {
    panic!("Expected point entity, found brush entity");
}

fn panic_expected_standard() {
    panic!("Expected standard surface, found Valve220 surface");
}

fn panic_expected_valve() {
    panic!("Expected Valve220 surface, found standard surface");
}

fn panic_unexpected_variant<T: std::fmt::Display>(err: T) {
    panic!("Unexpected error variant for {}", err);
}

// Parse successes

#[test]
fn parse_empty_map() {
    let map = parse(&b""[..]).unwrap();
    assert_eq!(map.entities.len(), 0);
}

#[test]
fn parse_empty_point_entity() {
    let map = parse(&b"{ }"[..]).unwrap();
    assert_eq!(map.entities.len(), 1);
    let ent = &map.entities[0];
    assert_eq!(ent.edict().len(), 0);

    if let Entity::Point(_) = ent {
    } else {
        panic_expected_point();
    }
}

#[test]
fn parse_point_entity_with_key_value() {
    let map = parse(
        &br#"
        {
            "classname" "light"
        }
    "#[..],
    )
    .unwrap();
    assert_eq!(map.entities.len(), 1);
    let ent = &map.entities[0];
    let edict = ent.edict();
    assert_eq!(edict.len(), 1);
    assert_eq!(
        edict.iter().next().unwrap(),
        (
            &CString::new("classname").unwrap(),
            &CString::new("light").unwrap()
        )
    );
}

#[test]
fn parse_standard_brush_entity() {
    let map = parse(
        &b"
        {
            {
                ( 1 2 3 ) ( 4 5 6 ) ( 7 8 9 ) TEXTURE1 0 0 0 1 1
            }
        }
    "[..],
    )
    .unwrap();
    assert_eq!(map.entities.len(), 1);
    let ent = &map.entities[0];

    if let Entity::Brush(_, brushes) = ent {
        assert_eq!(brushes.len(), 1);
        let brush = &brushes[0];
        assert_eq!(brush.len(), 1);
        let surface = &brush[0];
        assert_eq!(
            surface.half_space,
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]
        );
        assert_eq!(surface.texture, CString::new("TEXTURE1").unwrap());

        if let Alignment::Standard(base) = surface.alignment {
            assert_eq!(base.offset, [0.0, 0.0]);
            assert_eq!(base.rotation, 0.0);
            assert_eq!(base.scale, [1.0, 1.0]);
        } else {
            panic_expected_standard();
        }
    } else {
        panic_expected_brush();
    }
}

#[test]
fn parse_valve_brush_entity() {
    let map = parse(
        &b"
        {
        {
            ( 1 2 3 ) ( 4 5 6 ) ( 7 8 9 ) TEX2 [ 1 0 0 0 ] [ 0 1 0 0 ] 0 1 1
        }
        }
    "[..],
    )
    .unwrap();
    assert_eq!(map.entities.len(), 1);
    let ent = &map.entities[0];
    assert_eq!(ent.edict().len(), 0);

    if let Entity::Brush(_, brushes) = ent {
        assert_eq!(brushes.len(), 1);
        let brush = &brushes[0];
        assert_eq!(brush.len(), 1);
        let surface = &brush[0];
        assert_eq!(
            surface.half_space,
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]
        );
        assert_eq!(surface.texture, CString::new("TEX2").unwrap());

        if let Alignment::Valve220(base, axes) = surface.alignment {
            assert_eq!(base.offset, [0.0, 0.0]);
            assert_eq!(base.rotation, 0.0);
            assert_eq!(base.scale, [1.0, 1.0]);
            assert_eq!(axes, [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        } else {
            panic_expected_valve();
        }
    } else {
        panic_expected_brush();
    }
}

#[test]
fn parse_weird_numbers() {
    let map = parse(
        &b"
        { {
            ( 9E99 1E-9 -1.37e9 ) ( 12 -0 -100.7 ) ( 0e8 0E8 1.2e34 )
                T 0.25 0.25 45 2.0001 2.002
        } }
    "[..],
    )
    .unwrap();

    if let Entity::Brush(_, brushes) = &map.entities[0] {
        let brush = &brushes[0];
        let surface = &brush[0];

        assert_eq!(
            surface.half_space,
            [
                [9E99, 1E-9, -1.37e9],
                [12.0, 0.0, -100.7],
                [0.0, 0.0, 1.2e34]
            ]
        );

        if let Alignment::Standard(base) = surface.alignment {
            assert_eq!(base.offset, [0.25, 0.25]);
            assert_eq!(base.rotation, 45.0);
            assert_eq!(base.scale, [2.0001, 2.002]);
        } else {
            panic_expected_standard();
        }
    } else {
        panic_expected_brush();
    }
}

#[test]
fn parse_weird_textures() {
    let map = parse(
        &br#"
        { {
            ( 1 2 3 ) ( 4 5 6 ) ( 7 8 9 )
            {FENCE
            0 0 0 1 1

            ( 9 8 7 ) ( 6 5 4 ) ( 9 8 7 )
            "spaced out"
            0 0 0 1 1
        } }
    "#[..],
    )
    .unwrap();

    if let Entity::Brush(_, brushes) = &map.entities[0] {
        let surface1 = &(&brushes[0])[0];
        let surface2 = &(&brushes[0])[1];
        assert_eq!(surface1.texture, CString::new("{FENCE").unwrap());
        assert_eq!(surface2.texture, CString::new("spaced out").unwrap());
    } else {
        panic_expected_brush();
    }
}

// Parse errors

#[test]
fn parse_token_error() {
    let err = parse(&b"\""[..]).err().unwrap();
    if let qmap::result::Error::Lexer(line_err) = err {
        assert_eq!(u64::from(line_err.line_number.unwrap()), 1u64);
    } else {
        panic_unexpected_variant(err);
    }
}

#[test]
fn parse_io_error() {
    let reader = ErroringReader::new();
    let err = parse(reader).err().unwrap();
    if let qmap::result::Error::Io(_) = err {
    } else {
        panic_unexpected_variant(err);
    }
}

#[test]
fn parse_eof_error() {
    let err = parse(&b"{"[..]).err().unwrap();
    if let qmap::result::Error::Parser(line_err) = err {
        assert_eq!(line_err.line_number, None);
        assert!(line_err.message.contains("end-of-file"));
    } else {
        panic_unexpected_variant(err);
    }
}

#[test]
fn parse_closing_brace_error() {
    let err = parse(&b"}"[..]).err().unwrap();
    if let qmap::result::Error::Parser(line_err) = err {
        assert_eq!(u64::from(line_err.line_number.unwrap()), 1u64);
        assert!(line_err.message.contains("}"));
    } else {
        panic_unexpected_variant(err);
    }
}

#[test]
fn parse_missing_value() {
    let err = parse(&b"{\n \"classname\"\n }"[..]).err().unwrap();
    if let qmap::result::Error::Parser(line_err) = err {
        assert_eq!(u64::from(line_err.line_number.unwrap()), 3u64);
        assert!(line_err.message.contains("}"));
    } else {
        panic_unexpected_variant(err);
    }
}
