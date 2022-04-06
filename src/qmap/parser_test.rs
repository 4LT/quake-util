use crate::qmap;
use qmap::parser::parse;

// Parse successes

#[test]
fn parse_empty_map() {
    let ast = parse(&b""[..]).unwrap();
    assert_eq!(ast.entities.len(), 0);
}

// Parse errors

#[test]
fn parse_token_error() {
    let err = parse(&b"\""[..]).err().unwrap();
    if let qmap::result::Error::Lexer(_) = err {
    } else {
        panic!("Unexpected error variant for {}", err);
    }
}
