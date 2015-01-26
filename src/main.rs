fn to_benc (str: &str) -> String {
    let l1 = str.len();
    let tmp = format!("{}:", l1);
    let l2 = tmp.len();

    let mut s = String::with_capacity(l1 + l2);
    s.push_str(tmp.as_slice());
    s.push_str(str.as_slice());

    return s;
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::to_benc;

    #[test]
    fn test_to_benc() {
        assert_eq!("0:", to_benc(""));
        assert_eq!("4:test", to_benc("test"));
    }
}
