extern crate angstrom;
use std::collections::BTreeMap;
use angstrom::base::*;
use angstrom::bytes::*;

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub enum Benc<'a> {
    Nil,
    S (String),
    I (i64),
    L (BList<'a>),
    // TODO: These should be sorted by binary values of the keys. For now,
    //       it is unsorted.
    D (BDict<'a>)
}

pub type BList<'a> = Vec<Benc<'a>>;
pub type BDict<'a> = BTreeMap<Vec<u8>, Benc<'a>>;

pub trait BEncodable {
    fn benc_encode (&self) -> String;
}

// String implementations.

impl BEncodable for [u8] {
    fn benc_encode (&self) -> String {
        String::from_utf8(self.to_vec()).unwrap().benc_encode()
    }
}

impl BEncodable for Vec<u8> {
    fn benc_encode (&self) -> String {
        // String::from_ut8 takes its argument by value and using it here
        // would create ownership problems. This is the stupid way to do
        // it (and there certainly is a more idiomatic way to do it).
        self.as_slice().benc_encode()
    }
}

impl BEncodable for str {
    fn benc_encode (&self) -> String {
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
    fn benc_encode (&self) -> String {
        self.as_slice().benc_encode()
    }
}

// Integer implementations.

impl BEncodable for i64 {
    fn benc_encode (&self) -> String {
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
    fn benc_encode (&self) -> String {
        let mut tmp = String::from_str("l");

        for b in self.iter() {
            tmp.push_str(b.benc_encode().as_slice());
        }

        tmp.push_str("e");

        return tmp;
    }
}

impl<T: BEncodable> BEncodable for [T] {
    fn benc_encode (&self) -> String {
        let mut tmp = String::from_str("l");

        for b in self.iter() {
            tmp.push_str(b.benc_encode().as_slice());
        }

        tmp.push_str("e");

        return tmp;
    }
}

// Dictionary implementations.

impl<'a, T: BEncodable> BEncodable for BTreeMap<Vec<u8>, T> {
    fn benc_encode(&self) -> String {
        let mut tmp = String::from_str("d");

        for (key, value) in self.iter() {
            tmp.push_str(key.benc_encode().as_slice());
            tmp.push_str(value.benc_encode().as_slice());
        }

        tmp.push_str("e");

        return tmp;
    }
}

// BEnc implementations

impl<'a> BEncodable for Benc<'a> {
    fn benc_encode(&self) -> String {
        match *self {
            // TODO: replace "".benc_encode() with empty list.
            Benc::Nil      => "".benc_encode(),
            Benc::S(ref s) => s.benc_encode(),
            Benc::I(ref i) => i.benc_encode(),
            Benc::L(ref l) => l.benc_encode(),
            Benc::D(ref d) => d.benc_encode(),
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

pub fn parse_benc<'a>() -> Parser<'a, u8, Benc<'a>> {
    fn temp1<'b>() -> Parser<'b, u8, Benc<'b>> {
        parser_lazy_or(parse_bencoded_list, parse_bencoded_dictionary)
    }

    fn temp2<'b>() -> Parser<'b, u8, Benc<'b>> {
        parser_lazy_or(parse_string_to_benc, temp1)
    }

    parser_lazy_or(parse_i64_to_benc, temp2)
}

fn parse_i64_to_benc<'a>() -> Parser<'a, u8, Benc<'a>> {
    let p = parser_i64();
    let inbetween = parser_lift(Benc::I, p);
    let open = exactly(b'i');
    let close = exactly(b'e');

    parser_between(open, close, inbetween)
}

fn parse_string_to_benc<'a>() -> Parser<'a, u8, Benc<'a>> {
    let p = parse_bencoded_string();

    parser_lift(u8_to_benc, p)
}

fn parse_bencoded_list<'a>()
                            -> Parser<'a, u8, Benc<'a>> {
    let open = exactly(b'l');
    let close = exactly(b'e');

    fn temp<'a>(v: Vec<Benc>) -> Benc {
        match v.len() {
            0 => Benc::Nil,
            _ => Benc::L(v)
        }
    };

    let inbetween = parser_lift(temp, parser_many(parse_benc()));

    parser_between(open, close, inbetween)
}

fn parse_bencoded_dictionary<'a>() -> Parser<'a, u8, Benc<'a>> {
    let open = exactly(b'd');
    let close = exactly(b'e');

    let inbetween = parse_dictionary_pair(parse_benc());

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

fn parse_bencoded_string<'a>() -> Parser<'a, u8, Vec<u8>> {
    fn temp<'a>(n: i64) -> Parser<'a, u8, Vec<u8>> {
        parser_ignore_first(exactly(b':'),
                   parser_ntimes(n, any()))
    }

    let p = parser_i64();

    parser_bind(p, temp)
}

fn main() {
    println!("In progress.");
}

#[cfg(test)]
mod tests {
    use super::BEncodable;
    use std::collections::BTreeMap;
    use super::Benc;
    use angstrom::base::*;

#[test]
    fn test_str_benc_encode() {
        assert_eq!("0:", "".benc_encode());
        assert_eq!("4:test", "test".benc_encode());
        assert_eq!("15:ohmygodwhoisshe",
                "ohmygodwhoisshe".benc_encode());
    }

#[test]
    fn test_i64_benc_encode() {
        assert_eq!("i0e", 0.benc_encode());
        assert_eq!("i1e", 1.benc_encode());
        assert_eq!("i1000e", 1000.benc_encode());
        assert_eq!("i-1e", (-1).benc_encode());
    }

#[test]
    fn test_vec_benc_encode() {
        {
            let xs : Vec<i64> = Vec::new();
            assert_eq!("le", xs.benc_encode());
        }
        {
            let xs : Vec<i64> = vec![0,1,2,3];
            assert_eq!("li0ei1ei2ei3ee", xs.benc_encode());
        }
        {
            let xs = vec!["Hello,".to_string(),
                " ".to_string(),
                "World!".to_string()];
            let xs_benc_encode = "l6:Hello,1: 6:World!e";

            assert_eq!(xs_benc_encode, xs.benc_encode());
        }
    }

#[test]
    fn test_array_benc_encode() {
        {
            let ar = ["Hello,".to_string(),
                " ".to_string(),
                "World!".to_string()];
            let ar_benc_encode = "l6:Hello,1: 6:World!e";

            assert_eq!(ar_benc_encode, ar.benc_encode());
        }
    }

#[test]
    fn test_map_benc_encode() {
        {
            let map : BTreeMap<Vec<u8>, i64> = BTreeMap::new();

            assert_eq!(map.benc_encode(), "de");
        }
        {
            let mut map : BTreeMap<Vec<u8>, i64> = BTreeMap::new();
            map.insert(b"cow".to_vec(), 0);
            map.insert(b"dus".to_vec(), 10000);

            let map_benc_encode = "d3:cowi0e3:dusi10000ee";

            assert_eq!(map.benc_encode(), map_benc_encode);
        }
    }

#[test]
    fn test_benc_benc_encode() {
        {
            let b = Benc::S("Hello".to_string());
            assert_eq!(b.benc_encode(), "5:Hello");
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
        let parser = super::parse_string_to_benc();

        let slice1 = b"5:Hello";
        let result = "Hello";

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
                = super::parse_bencoded_list();

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
                = super::parse_bencoded_list();

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
            = super::parse_bencoded_dictionary();

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
    fn test_parse_benc() {
        let parser = super::parse_benc();

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
}
