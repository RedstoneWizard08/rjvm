extern crate rjvm;

use rjvm::class_file_field::{ClassFileField, FieldConstantValue};
use rjvm::field_flags::FieldFlags;
use tracing::{event, info, span, Level};

mod utils;

#[test]
fn can_read_constants() {
    utils::setup_tracing();
    let class = utils::read_class_from_file("Constants");
    // event!(Level::INFO, %class, "read class file");
    info!("read class file {}", class);
    assert_eq!(
        vec!(
            ClassFileField {
                flags: FieldFlags::PUBLIC | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "AN_INT".to_string(),
                type_descriptor: "I".to_string(),
                constant_value: Some(FieldConstantValue::Int(2023)),
            },
            ClassFileField {
                flags: FieldFlags::PROTECTED | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "A_FLOAT".to_string(),
                type_descriptor: "F".to_string(),
                constant_value: Some(FieldConstantValue::Float(20.23)),
            },
            ClassFileField {
                flags: FieldFlags::PRIVATE | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "A_LONG".to_string(),
                type_descriptor: "J".to_string(),
                constant_value: Some(FieldConstantValue::Long(2023)),
            },
            ClassFileField {
                flags: FieldFlags::PUBLIC | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "A_DOUBLE".to_string(),
                type_descriptor: "D".to_string(),
                constant_value: Some(FieldConstantValue::Double(20.23)),
            },
            ClassFileField {
                flags: FieldFlags::PUBLIC | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "A_STRING".to_string(),
                type_descriptor: "Ljava/lang/String;".to_string(),
                constant_value: Some(FieldConstantValue::String("2023".to_string())),
            }
        ),
        class.fields
    );
}
