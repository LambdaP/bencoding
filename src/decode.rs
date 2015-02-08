use encode::*;
use angstrom::base::*;
use angstrom::bytes::*;
use std::collections::BTreeMap;

pub type BParser<'a> = Parser<'a, u8, Benc<'a>>;

pub fn decode_benc<'a>() -> BParser<'a> {
    fn temp1<'b>() -> BParser<'b> {
        parser_lazy_or(decode_list, decode_dictionary)
    }

    fn temp2<'b>() -> BParser<'b> {
        parser_lazy_or(decode_i64, temp1)
    }

    parser_lazy_or(decode_string, temp2)
}

fn decode_string<'a>() -> BParser<'a> {
    let p = decode_bencoded_string();

    fn string_or_nil<'a>(v: Vec<u8>) -> Benc<'a> {
        let s = String::from_utf8(v).unwrap();
    
        match s.len() {
            0 => Benc::Nil,
            _ => Benc::S(s)
        }
    }

    parser_lift(string_or_nil, p)
}

fn decode_i64<'a>() -> BParser<'a> {
    let open = exactly(b'i');
    let close = exactly(b'e');

    let inbetween = parser_lift(Benc::I, parser_i64());

    parser_between(open, close, inbetween)
}

fn decode_list<'a>() -> BParser<'a> {
    let open = exactly(b'l');
    let close = exactly(b'e');

    fn list_or_nil<'a>(v: Vec<Benc>) -> Benc {
        match v.len() {
            0 => Benc::Nil,
            _ => Benc::L(v)
        }
    };

    let inbetween = parser_lift(list_or_nil, parser_many(decode_benc()));

    parser_between(open, close, inbetween)
}

fn decode_dictionary<'a>() -> BParser<'a> {
    let open = exactly(b'd');
    let close = exactly(b'e');

    fn pair_vect_to_benc<'a>(v: Vec<(Vec<u8>, Benc<'a>)>) -> Benc<'a> {
        let mut dict : BDict<'a> = BTreeMap::new();
    
        for (key, val) in v.into_iter() {
            dict.insert(key, val);
        }

        match dict.len() {
            0 => Benc::Nil,
            _ => Benc::D(dict)
        }
    }

    let inbetween =
        parser_lift(pair_vect_to_benc,
                    parser_many(parser_and(decode_bencoded_string(),
                                           decode_benc())));

    parser_between(open, close, inbetween)
}

fn decode_bencoded_string<'a>() -> Parser<'a, u8, Vec<u8>> {
    fn ignore_length<'b>(n: i64) -> Parser<'b, u8, Vec<u8>> {
        parser_ignore_first(exactly(b':'),
                   parser_ntimes(n, any()))
    }

    let p = parser_i64();

    parser_bind(p, ignore_length)
}

#[test]
fn test_decode_bencoded_string() {
    {
        let parser = decode_bencoded_string();

        let slice1 = b"5:Hello";
        let result = b"Hello";

        match parser.parse(slice1) {
            None => assert!(false),
                 Some(v) => assert_eq!(v, result.to_vec())
        }

        let slice2 = &[0, 58, 0, 1, 2, 3];
        let slice3 = b":Hello";
        let slice4 = b"5:Hell";
        let slice5 = b"5:Hello World!";

        assert_eq!(parser.parse(slice2), None);
        assert_eq!(parser.parse(slice3), None);
        assert_eq!(parser.parse(slice4), None);
        assert_eq!(parser.parse(slice5), None);
    }
    {
        let parser = decode_bencoded_string();

        let slice1 = b"1:H";
        let result = b"H";

        match parser.parse(slice1) {
            None => assert!(false),
                 Some(v) => assert_eq!(v, result.to_vec())
        }
    }
    {
        let parser = decode_bencoded_string();

        let slice1 = b"0:";
        let result = b"";

        match parser.parse(slice1) {
            None => assert!(false),
                 Some(v) => assert_eq!(v, result.to_vec())
        }
    }
}

#[test]
fn test_decode_string() {
    let parser = decode_string();

    let slice1 = b"5:Hello";
    let result = "Hello";

    match parser.parse(slice1) {
        Some(Benc::S(x)) => assert_eq!(x, result),
            _ => assert!(false)
    }
}

#[test]
fn test_parse_u8_to_benc() {
    let parser = decode_i64();

    let slice1 = b"i0e";
    let slice2 = b"i777e";
    let slice3 = b"i1234567890e";

    match parser.parse(slice1) {
        Some(Benc::I(x)) => assert_eq!(x, 0),
            _ => assert!(false)
    }
    match parser.parse(slice2) {
        Some(Benc::I(x)) => assert_eq!(x, 777),
            _ => assert!(false)
    }
    match parser.parse(slice3) {
        Some(Benc::I(x)) => assert_eq!(x, 1234567890),
            _ => assert!(false)
    }

    let slice4 = b"ie";
    let slice5 = b"i01e";
    let slice6 = b"5:Hello";

    assert_eq!(parser.parse(slice4), None);
    assert_eq!(parser.parse(slice5), None);
    assert_eq!(parser.parse(slice6), None);
}

#[test]
fn test_decode_list() {
    {
        let parser
            = decode_list();

        let slice1 = b"le";
        let slice2 = b"li0ee";
        let slice3 = b"li0ei1ei2ee";

        let result2 = vec![Benc::I(0)];
        let result3 = vec![Benc::I(0), Benc::I(1), Benc::I(2)];

        match parser.parse(slice1) {
            Some(Benc::Nil) => assert!(true),
                _ => assert!(false)
        }
        match parser.parse(slice2) {
            Some(Benc::L(v)) => assert_eq!(v, result2),
                _ => assert!(false)
        }
        match parser.parse(slice3) {
            Some(Benc::L(v)) => assert_eq!(v, result3),
                _ => assert!(false)
        }

        // TODO: add failing tests.
    }
    {
        let parser = decode_list();

        assert!(true);

        let slice1 = b"l5:Hello6:World!e";

        let result1 = vec![Benc::S("Hello".to_string()),
            Benc::S("World!".to_string())];

        match parser.parse(slice1) {
            Some(Benc::L(v)) => assert_eq!(v, result1),
                _ => assert!(false)
        }
    }
}

#[test]
fn test_decode_dictionary() {
    let parser = decode_dictionary();

    let slice1 = b"d1:Ai1e1:Bi2e1:Ci3ee";

    let mut result1 : BTreeMap<Vec<u8>, Benc> = BTreeMap::new();
    result1.insert(b"A".to_vec(), Benc::I(1));
    result1.insert(b"B".to_vec(), Benc::I(2));
    result1.insert(b"C".to_vec(), Benc::I(3));

    match parser.parse(slice1) {
        Some(Benc::D(d)) => assert_eq!(d, result1),
            _ => assert!(false)
    }
}

#[test]
fn test_decode_benc() {
    let parser = decode_benc();

    let slice1 = b"i0e";
    match parser.parse(slice1) {
        Some(Benc::I(x)) => assert_eq!(x, 0),
            _ => assert!(false)
    }

    let slice2 = b"5:Hello";
    let result2 = "Hello";
    match parser.parse(slice2) {
        Some(Benc::S(x)) => assert_eq!(x, result2),
            _ => assert!(false)
    }

    let slice3 = b"li0ei1ee";
    let result3 = vec![Benc::I(0), Benc::I(1)];
    match parser.parse(slice3) {
        Some(Benc::L(v)) => assert_eq!(v, result3),
            _ => assert!(false)
    }

    let slice4 = b"d1:Ai1e1:Bi2ee";
    let mut result4 : BTreeMap<Vec<u8>, Benc> = BTreeMap::new();
    result4.insert(b"A".to_vec(), Benc::I(1));
    result4.insert(b"B".to_vec(), Benc::I(2));
    match parser.parse(slice4) {
        Some(Benc::D(d)) => assert_eq!(d, result4),
            _ => assert!(false)
    }

    let slice5 = b"li0e5:Helloe";
    let result5 = vec![Benc::I(0), Benc::S("Hello".to_string())];
    match parser.parse(slice5) {
        Some(Benc::L(v)) => assert_eq!(v, result5),
            _ => assert!(false)
    }

    let slice6 = b"d1:Ali0ei1ee1:B5:Helloe";
    let mut result6 : BTreeMap<Vec<u8>, Benc> = BTreeMap::new();
    result6.insert(b"A".to_vec(), Benc::L(vec![Benc::I(0), Benc::I(1)]));
    result6.insert(b"B".to_vec(), Benc::S("Hello".to_string()));
    match parser.parse(slice6) {
        Some(Benc::D(d)) => assert_eq!(d, result6),
            _ => assert!(false)
    }

}
