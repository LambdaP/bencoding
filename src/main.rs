use std::collections::BTreeMap;

trait BEncodable {
    fn to_benc (&self) -> String;
}

impl BEncodable for str {
    fn to_benc (&self) -> String {
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
    fn to_benc (&self) -> String {
        self.as_slice().to_benc()
    }
}

impl BEncodable for i64 {
    fn to_benc (&self) -> String {
        return format!("i{}e", self);
    }
}

impl<T: BEncodable> BEncodable for Vec<T> {
    fn to_benc (&self) -> String {
        let mut tmp = String::from_str("l");

        for b in self.iter() {
            tmp.push_str(b.to_benc().as_slice());
        }

        tmp.push_str("e");

        return tmp;
    }
}

impl<'a, T: BEncodable> BEncodable for BTreeMap<&'a str, T> {
    fn to_benc(&self) -> String {
        let mut tmp = String::from_str("d");

        for (key, value) in self.iter() {
            tmp.push_str(key.to_benc().as_slice());
            tmp.push_str(value.to_benc().as_slice());
        }

        tmp.push_str("e");

        return tmp;
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::BEncodable;
    use std::collections::BTreeMap;

#[test]
    fn test_str_to_benc() {
        assert_eq!("0:", "".to_benc());
        assert_eq!("4:test", "test".to_benc());
    }

#[test]
    fn test_i64_to_benc() {
        assert_eq!("i0e", 0.to_benc());
        assert_eq!("i1e", 1.to_benc());
        assert_eq!("i1000e", 1000.to_benc());
        assert_eq!("i-1e", (-1).to_benc());
    }

#[test]
    fn test_vec_to_benc() {
        {
            let xs : Vec<i64> = Vec::new();
            assert_eq!("le", xs.to_benc());
        }
        {
            let xs : Vec<i64> = vec![0,1,2,3];
            assert_eq!("li0ei1ei2ei3ee", xs.to_benc()); 
        }
        {
            let xs = vec!["Hello,".to_string(),
                          " ".to_string(),
                          "World!".to_string()];
            let xs_to_benc = "l6:Hello,1: 6:World!e";

            assert_eq!(xs_to_benc, xs.to_benc());
        }
    }

#[test]
    fn test_map_to_benc() {
        {
            let mut map : BTreeMap<&str, i64> = BTreeMap::new();
            map.insert("cow", 0);
            map.insert("dus", 10000);

            let map_to_benc = "d3:cowi0e3:dusi10000ee";

            assert_eq!(map.to_benc(), map_to_benc);
        }
    }
}
