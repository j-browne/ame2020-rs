use crate::{AmeError, Iter};
use std::io::{self, Cursor};

// if the file is empty, that's not an error, there are just no items
#[test]
fn empty() {
    let reader = Cursor::new(include_str!("tests/empty"));
    let mut iter = Iter::new(reader);
    assert!(iter.next().is_none());
}

#[test]
fn only_preamble() {
    let reader = Cursor::new(include_str!("tests/only_preamble"));
    let mut iter = Iter::new(reader);
    assert!(iter.next().is_none());
}

#[test]
fn only_preamble_and_headers() {
    let reader = Cursor::new(include_str!("tests/only_preamble_and_headers"));
    let mut iter = Iter::new(reader);
    assert!(iter.next().is_none());
}

// if there's no preamble, the parser doesn't know to read data
#[test]
fn no_preamble() {
    let reader = Cursor::new(include_str!("tests/no_preamble"));
    let mut iter = Iter::new(reader);
    assert!(iter.next().is_none());
}

// stuff before the preamble is ok
#[test]
fn pre_preamble() {
    let reader = Cursor::new(include_str!("tests/pre_preamble"));
    let mut iter = Iter::new(reader);
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
}

// too many page feeds ('1') is fine. there just needs to be at least 2
#[test]
fn too_many_page_feeds() {
    let reader = Cursor::new(include_str!("tests/too_many_page_feeds"));
    let mut iter = Iter::new(reader);
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
}

// the headers can take multiple lines
// the data starts with a line feed ('0')
#[test]
fn extra_headers() {
    let reader = Cursor::new(include_str!("tests/extra_headers"));
    let mut iter = Iter::new(reader);
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
}

#[test]
fn single() {
    let reader = Cursor::new(include_str!("tests/single"));
    let mut iter = Iter::new(reader);
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
}

// make sure you get the right error when the line is too short
#[test]
fn too_short_line() {
    let reader = Cursor::new(include_str!("tests/too_short_line"));
    let mut iter = Iter::new(reader);
    assert_eq!(iter.next().unwrap(), Err(AmeError::TooShortLine));
    assert_eq!(iter.next().unwrap(), Err(AmeError::TooShortLine));
    assert!(iter.next().is_none());
}

// the input for this test has multi-byte chars
#[test]
fn str_index() {
    // the char spans a slice boundary, so we get an indexing error
    let reader = Cursor::new(include_str!("tests/str_index_1"));
    let mut iter = Iter::new(reader);
    assert_eq!(iter.next().unwrap(), Err(AmeError::StrIndex));
    assert!(iter.next().is_none());

    // the char is within a slice, so we get a parsing error
    let reader = Cursor::new(include_str!("tests/str_index_2"));
    let mut iter = Iter::new(reader);
    assert!(matches!(iter.next().unwrap(), Err(AmeError::ParseFloat(_))));
    assert!(iter.next().is_none());
}

// the input for this test has a non-utf8 byte
#[test]
fn non_utf8() {
    let reader = Cursor::new(include_bytes!("tests/non_utf8"));
    let mut iter = Iter::new(reader);
    assert_eq!(
        iter.next().unwrap(),
        Err(AmeError::Io(io::ErrorKind::InvalidData))
    );
}

// This test should be able to open "src", but since it is a directory, reading from it should be
// an error.
#[test]
fn io_error() {
    use std::{fs::File, io::BufReader};

    let reader = File::open("src").unwrap();
    let reader = BufReader::new(reader);
    let mut iter = Iter::new(reader);
    assert!(matches!(iter.next().unwrap(), Err(AmeError::Io(_))));
}

#[test]
fn parse_error() {
    // fails to parse an int in n
    let reader = Cursor::new(include_str!("tests/parse_int_error_1"));
    let mut iter = Iter::new(reader);
    assert!(matches!(iter.next().unwrap(), Err(AmeError::ParseInt(_))));

    // fails to parse an int in z
    let reader = Cursor::new(include_str!("tests/parse_int_error_2"));
    let mut iter = Iter::new(reader);
    assert!(matches!(iter.next().unwrap(), Err(AmeError::ParseInt(_))));

    // fails to parse an int in the first part of mass
    let reader = Cursor::new(include_str!("tests/parse_int_error_3"));
    let mut iter = Iter::new(reader);
    assert!(matches!(iter.next().unwrap(), Err(AmeError::ParseInt(_))));

    // fails to parse a float in the mass excess mean
    let reader = Cursor::new(include_str!("tests/parse_float_error_1"));
    let mut iter = Iter::new(reader);
    assert!(matches!(iter.next().unwrap(), Err(AmeError::ParseFloat(_))));

    // fails to parse a float in the mass excess uncertainty
    let reader = Cursor::new(include_str!("tests/parse_float_error_2"));
    let mut iter = Iter::new(reader);
    assert!(matches!(iter.next().unwrap(), Err(AmeError::ParseFloat(_))));
}

#[test]
fn multi() {
    let reader = Cursor::new(include_str!("tests/multi"));
    let iter = Iter::new(reader);
    assert!(matches!(iter.collect::<Result<Vec<_>, _>>(), Ok(_)));
}
