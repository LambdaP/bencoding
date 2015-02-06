extern crate angstrom;
use std::collections::BTreeMap;
use angstrom::base::*;
use angstrom::bytes::*;

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
enum Benc<'a> {
    Nil,
    S (String),
    I (i64),
    L (BList<'a>),
    // TODO: These should be sorted by binary values of the keys. For now,
    //       it is unsorted.
    D (BDict<'a>)
}

type BList<'a> = Vec<Benc<'a>>;
type BDict<'a> = BTreeMap<Vec<u8>, Benc<'a>>;

trait BEncodable {
    fn serialize (&self) -> String;
}

// String implementations.

impl BEncodable for [u8] {
    fn serialize (&self) -> String {
        String::from_utf8(self.to_vec()).unwrap().serialize()
    }
}

impl BEncodable for Vec<u8> {
    fn serialize (&self) -> String {
        // String::from_ut8 takes its argument by value and using it here
        // would create ownership problems. This is the stupid way to do
        // it (and there certainly is a more idiomatic way to do it).
        self.as_slice().serialize()
    }
}

impl BEncodable for str {
    fn serialize (&self) -> String {
        let l1 = self.len();
        let tmp = format!("{}:", l1);
        let l2 = tmp.len();

        let mut s = String::with_capacity(l1 + l2);
        s.push_str(tmp.as_slice());
        s.push_str(self);

        return s;
    }
}

impl BEncodable for String {
    fn serialize (&self) -> String {
        self.as_slice().serialize()
    }
}

// Integer implementations.

impl BEncodable for i64 {
    fn serialize (&self) -> String {
        return format!("i{}e", self);
    }
}

// List implementations.

// TODO: find a way to factor duplicate code.
//       This should be doable with a function that operates on an
//       Iterator rather than on specific types, but at the moment I can't
//       find how to specify something like
//       impl<T: BEncodable> for Iterator<T>

impl<T: BEncodable> BEncodable for Vec<T> {
    fn serialize (&self) -> String {
        let mut tmp = String::from_str("l");

        for b in self.iter() {
            tmp.push_str(b.serialize().as_slice());
        }

        tmp.push_str("e");

        return tmp;
    }
}

impl<T: BEncodable> BEncodable for [T] {
    fn serialize (&self) -> String {
        let mut tmp = String::from_str("l");

        for b in self.iter() {
            tmp.push_str(b.serialize().as_slice());
        }

        tmp.push_str("e");

        return tmp;
    }
}

// Dictionary implementations.

impl<'a, T: BEncodable> BEncodable for BTreeMap<Vec<u8>, T> {
    fn serialize(&self) -> String {
        let mut tmp = String::from_str("d");

        for (key, value) in self.iter() {
            tmp.push_str(key.serialize().as_slice());
            tmp.push_str(value.serialize().as_slice());
        }

        tmp.push_str("e");

        return tmp;
    }
}

// BEnc implementations

impl<'a> BEncodable for Benc<'a> {
    fn serialize(&self) -> String {
        match *self {
            // TODO: replace "".serialize() with empty list.
            Benc::Nil      => "".serialize(),
            Benc::S(ref s) => s.serialize(),
            Benc::I(ref i) => i.serialize(),
            Benc::L(ref l) => l.serialize(),
            Benc::D(ref d) => d.serialize(),
        }
    }
}

fn u8_to_benc<'a>(v: Vec<u8>) -> Benc<'a> {
    let s = String::from_utf8(v).unwrap();

    match s.len() {
        0 => Benc::Nil,
        _ => Benc::S(s)
    }
}

fn parse_i64_to_benc<'a>() -> Parser<'a, u8, Benc<'a>> {
    let p = parser_i64();
    let inbetween = parser_lift(Benc::I, p);
    let open = exactly(b'i');
    let close = exactly(b'e');

    parser_between(open, close, inbetween)
}

fn parse_bencoded_string<'a>() -> Parser<'a, u8, Vec<u8>> {
    fn temp<'a>(n: i64) -> Parser<'a, u8, Vec<u8>> {
        parser_ignore_first(exactly(b':'),
                   parser_ntimes(n, any()))
    }

    let p = parser_i64();

    parser_bind(p, temp)
}

fn parse_bencoded_list<'a>(p: Parser<'a, u8, Benc<'a>>)
                            -> Parser<'a, u8, Benc<'a>> {
    let open = exactly(b'l');
    let close = exactly(b'e');

    fn temp<'a>(v: Vec<Benc>) -> Benc {
        match v.len() {
            0 => Benc::Nil,
            _ => Benc::L(v)
        }
    };

    let inbetween = parser_lift(temp, parser_many(p));

    parser_between(open, close, inbetween)
}

fn pair_vect_to_benc<'a>(v: Vec<(Vec<u8>, Benc<'a>)>) -> Benc<'a> {
    let mut dict : BDict<'a> = BTreeMap::new();

    for (key, val) in v.into_iter() {
        dict.insert(key, val);
    }

    Benc::D(dict)
}

fn parse_dictionary_pair<'a>(p: Parser<'a, u8, Benc<'a>>)
                                -> Parser<'a, u8, Benc<'a>> {
    parser_lift(pair_vect_to_benc,
                parser_many(parser_and(parse_bencoded_string(), p)))
}

fn parse_bencoded_dictionary<'a>(p: Parser<'a, u8, Benc<'a>>)
                                    -> Parser<'a, u8, Benc<'a>> {
    let open = exactly(b'd');
    let close = exactly(b'e');

    let inbetween = parse_dictionary_pair(p);

    parser_between(open, close, inbetween)
}

fn parse_string_to_benc<'a>() -> Parser<'a, u8, Benc<'a>> {
    let p = parse_bencoded_string();

    parser_lift(u8_to_benc, p)
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::BEncodable;
    use std::collections::BTreeMap;
    use super::Benc;
    use super::*;
    use angstrom::base::*;

#[test]
    fn test_str_serialize() {
        assert_eq!("0:", "".serialize());
        assert_eq!("4:test", "test".serialize());
        assert_eq!("15:ohmygodwhoisshe",
                "ohmygodwhoisshe".serialize());
    }

#[test]
    fn test_i64_serialize() {
        assert_eq!("i0e", 0.serialize());
        assert_eq!("i1e", 1.serialize());
        assert_eq!("i1000e", 1000.serialize());
        assert_eq!("i-1e", (-1).serialize());
    }

#[test]
    fn test_vec_serialize() {
        {
            let xs : Vec<i64> = Vec::new();
            assert_eq!("le", xs.serialize());
        }
        {
            let xs : Vec<i64> = vec![0,1,2,3];
            assert_eq!("li0ei1ei2ei3ee", xs.serialize());
        }
        {
            let xs = vec!["Hello,".to_string(),
                " ".to_string(),
                "World!".to_string()];
            let xs_serialize = "l6:Hello,1: 6:World!e";

            assert_eq!(xs_serialize, xs.serialize());
        }
    }

#[test]
    fn test_array_serialize() {
        {
            let ar = ["Hello,".to_string(),
                " ".to_string(),
                "World!".to_string()];
            let ar_serialize = "l6:Hello,1: 6:World!e";

            assert_eq!(ar_serialize, ar.serialize());
        }
    }

#[test]
    fn test_map_serialize() {
        {
            let map : BTreeMap<Vec<u8>, i64> = BTreeMap::new();

            assert_eq!(map.serialize(), "de");
        }
        {
            let mut map : BTreeMap<Vec<u8>, i64> = BTreeMap::new();
            map.insert(b"cow".to_vec(), 0);
            map.insert(b"dus".to_vec(), 10000);

            let map_serialize = "d3:cowi0e3:dusi10000ee";

            assert_eq!(map.serialize(), map_serialize);
        }
    }

#[test]
    fn test_benc_serialize() {
        {
            let b = Benc::S("Hello".to_string());
            assert_eq!(b.serialize(), "5:Hello");
        }
    }

#[test]
    fn test_parse_bencoded_string() {
        {
            let parser = super::parse_bencoded_string();

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
            let parser = super::parse_bencoded_string();

            let slice1 = b"1:H";
            let result = b"H";

            match parser.parse(slice1) {
                None => assert!(false),
                     Some(v) => assert_eq!(v, result.to_vec())
            }
        }
        {
            let parser = super::parse_bencoded_string();

            let slice1 = b"0:";
            let result = b"";

            match parser.parse(slice1) {
                None => assert!(false),
                     Some(v) => assert_eq!(v, result.to_vec())
            }
        }
    }

#[test]
    fn test_parse_string_to_benc() {
        let slice1 = b"5:Hello";
        let result = "Hello";

        let parser = super::parse_string_to_benc();

        match parser.parse(slice1) {
            Some(Benc::S(x)) => assert_eq!(x, result),
            _ => assert!(false)
        }
    }

#[test]
    fn test_parse_u8_to_benc() {
        let parser = super::parse_i64_to_benc();

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
    fn test_parse_bencoded_list() {
        {
            let parser
                = super::parse_bencoded_list(super::parse_i64_to_benc());

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
            let parser
                = super::parse_bencoded_list(super::parse_string_to_benc());

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
    fn test_parse_bencoded_dictionary() {
        let parser
            = super::parse_bencoded_dictionary(super::parse_i64_to_benc());

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
}
