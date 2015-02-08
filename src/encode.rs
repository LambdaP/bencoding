use std::collections::BTreeMap;

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

// ## String implementations

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

// ## Integer implementations.

impl BEncodable for i64 {
    fn benc_encode (&self) -> String {
        return format!("i{}e", self);
    }
}

// ## List implementations.

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

// ## Dictionary implementations

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

// ## BEnc implementations

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
