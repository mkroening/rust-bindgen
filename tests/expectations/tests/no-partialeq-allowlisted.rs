#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct NoPartialEq {
    pub i: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_NoPartialEq() {
    assert_eq!(
        ::std::mem::size_of::<NoPartialEq>(),
        4usize,
        concat!("Size of: ", stringify!(NoPartialEq))
    );
    assert_eq!(
        ::std::mem::align_of::<NoPartialEq>(),
        4usize,
        concat!("Alignment of ", stringify!(NoPartialEq))
    );
    fn test_field_i() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<NoPartialEq>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).i) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(NoPartialEq),
                "::",
                stringify!(i)
            )
        );
    }
    test_field_i();
}