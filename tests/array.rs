use logix_type::LogixType;

#[test]
fn array_basics() {
    let array = [1, 2, 3];
    assert_eq!(array.map(|v| v * 2), vec![2, 4, 6].as_slice());
    assert_eq!(array.first(), Some(&1));
    assert_eq!(AsRef::<[i32]>::as_ref(&array), [1, 2, 3].as_slice());
    assert_eq!(format!("{array:?}"), "[1, 2, 3]");
    assert_eq!(array, [1, 2, 3]);
    assert_eq!(array, [1, 2, 3].as_slice());
    assert_eq!(<[i32; 3]>::default_value(), None);
}

#[test]
fn vec_basics() {
    assert_eq!(Vec::<i32>::default_value(), None);
}
