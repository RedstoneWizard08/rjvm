extern crate rjvm;

use rjvm::class_file::ClassFile;
use rjvm::class_file_field::ClassFileField;
use rjvm::class_file_method::ClassFileMethod;
use rjvm::field_flags::FieldFlags;
use rjvm::method_flags::MethodFlags;
use rjvm::{class_access_flags::ClassAccessFlags, class_file_version::ClassFileVersion};

mod utils;

#[test]
fn can_read_pojo_class_file() {
    utils::setup_tracing();
    let class = utils::read_class_from_file("Complex");
    println!("Read class file: {}", class);
    assert_eq!(ClassFileVersion::Jdk6, class.version);
    assert_eq!(
        ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
        class.flags
    );
    assert_eq!("rjvm/Complex", class.name);
    assert_eq!("java/lang/Object", class.superclass);
    assert_eq!(
        vec!("java/lang/Cloneable", "java/io/Serializable"),
        class.interfaces
    );

    check_fields(&class);
    check_methods(&class);
}

fn check_fields(class: &ClassFile) {
    assert_eq!(
        vec!(
            ClassFileField {
                flags: FieldFlags::PRIVATE | FieldFlags::FINAL,
                name: "real".to_string(),
                type_descriptor: "D".to_string(),
                constant_value: None,
            },
            ClassFileField {
                flags: FieldFlags::PRIVATE | FieldFlags::FINAL,
                name: "imag".to_string(),
                type_descriptor: "D".to_string(),
                constant_value: None,
            }
        ),
        class.fields
    );
}

fn check_methods(class: &ClassFile) {
    assert_eq!(5, class.methods.len());
    check_method(&class.methods[0], MethodFlags::PUBLIC, "<init>", "(D)V");
    check_method(&class.methods[1], MethodFlags::PUBLIC, "<init>", "(DD)V");
    check_method(&class.methods[2], MethodFlags::PUBLIC, "getReal", "()D");
    check_method(&class.methods[3], MethodFlags::PUBLIC, "getImag", "()D");
    check_method(&class.methods[4], MethodFlags::PUBLIC, "abs", "()D");
}

fn check_method(method: &ClassFileMethod, flags: MethodFlags, name: &str, type_descriptor: &str) {
    assert_eq!(method.flags, flags);
    assert_eq!(method.name, name);
    assert_eq!(method.type_descriptor, type_descriptor);
}
