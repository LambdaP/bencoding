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

impl BEncodable for i64 {
    fn to_benc (&self) -> String {
        return format!("i{}e", self);
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::BEncodable;

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
}
