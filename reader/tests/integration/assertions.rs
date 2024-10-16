extern crate rjvm_reader;

use rjvm_reader::{class::file::method::ClassFileMethod, method_flags::MethodFlags};

pub fn check_method(
    method: &ClassFileMethod,
    flags: MethodFlags,
    name: &str,
    type_descriptor: &str,
) {
    assert_eq!(method.flags, flags);
    assert_eq!(method.name, name);
    assert_eq!(method.type_descriptor, type_descriptor);
}
