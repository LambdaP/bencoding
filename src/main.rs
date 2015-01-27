use std::collections::BTreeMap;

enum Benc<'a> {
    S (String),
    I (i64),
    L (BList<'a>),
    D (BDict<'a>)
}

type BList<'a> = Vec<Benc<'a>>;
type BDict<'a> = BTreeMap<&'a str, Benc<'a>>;

trait BEncodable {
    fn serialize (&self) -> String;
}

// String implementations.

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

// TODO: find a way to factorise duplicate code.
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

impl<'a, T: BEncodable> BEncodable for BTreeMap<&'a str, T> {
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
            Benc::S(ref s) => s.serialize(),
            Benc::I(ref i) => i.serialize(),
            Benc::L(ref l) => l.serialize(),
            Benc::D(ref d) => d.serialize(),
        }
    }
}

// Decoding

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::BEncodable;
    use std::collections::BTreeMap;
    use super::Benc;

#[test]
    fn test_str_serialize() {
        assert_eq!("0:", "".serialize());
        assert_eq!("4:test", "test".serialize());
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
            let map : BTreeMap<&str, i64> = BTreeMap::new();

            assert_eq!(map.serialize(), "de");
        }
        {
            let mut map : BTreeMap<&str, i64> = BTreeMap::new();
            map.insert("cow", 0);
            map.insert("dus", 10000);

            let map_serialize = "d3:cowi0e3:dusi10000ee";

            assert_eq!(map.serialize(), map_serialize);
        }
    }

#[test]
    fn test_Benc_serialize() {
        {
            let b = Benc::S("Hello".to_string());
            assert_eq!(b.serialize(), "5:Hello");
        }
    }
}
