#[cfg(test)]
use encode::*;
use std::collections::BTreeMap;

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
