use std::{fs::File, io::Read, path::Path};

use result::prelude::*;
use tracing::warn;

use crate::attribute::Attribute;
use crate::class_file_field::{ClassFileField, FieldConstantValue};
use crate::class_file_method::ClassFileMethod;
use crate::class_reader_error::ClassReaderError::InvalidClassData;
use crate::field_flags::FieldFlags;
use crate::method_flags::MethodFlags;
use crate::{
    buffer::Buffer,
    class_access_flags::ClassAccessFlags,
    class_file::ClassFile,
    class_file_version::ClassFileVersion,
    class_reader_error::{ClassReaderError, Result},
    constant_pool::ConstantPoolEntry,
};

struct ClassFileReader<'a> {
    buffer: Buffer<'a>,
    class_file: ClassFile,
}

impl<'a> ClassFileReader<'a> {
    fn new(data: &[u8]) -> ClassFileReader {
        ClassFileReader {
            buffer: Buffer::new(data),
            class_file: Default::default(),
        }
    }

    fn read(mut self) -> Result<ClassFile> {
        self.check_magic_number()?;
        self.read_version()?;
        self.read_constants()?;
        self.read_access_flags()?;
        self.class_file.name = self.read_class_reference()?;
        self.class_file.superclass = self.read_class_reference()?;
        self.read_interfaces()?;
        self.read_fields()?;
        self.read_methods()?;

        Ok(self.class_file)
    }

    fn check_magic_number(&mut self) -> Result<()> {
        match self.buffer.read_u32() {
            Ok(0xCAFEBABE) => Ok(()),
            Ok(_) => Err(InvalidClassData("invalid magic number".to_owned())),
            Err(err) => Err(err),
        }
    }

    fn read_version(&mut self) -> Result<()> {
        let minor_version = self.buffer.read_u16()?;
        let major_version = self.buffer.read_u16()?;

        self.class_file.version = ClassFileVersion::from(major_version, minor_version)?;
        Ok(())
    }

    fn read_constants(&mut self) -> Result<()> {
        let constants_count = self.buffer.read_u16()? - 1;
        let mut i = 0;
        while i < constants_count {
            let tag = self.buffer.read_u8()?;
            let constant = match tag {
                1 => self.read_utf8_constant()?,
                3 => self.read_int_constant()?,
                4 => self.read_float_constant()?,
                5 => {
                    i += 1;
                    self.read_long_constant()?
                }
                6 => {
                    i += 1;
                    self.read_double_constant()?
                }
                7 => self.read_class_reference_constant()?,
                8 => self.read_string_reference_constant()?,
                9 => self.read_field_reference_constant()?,
                10 => self.read_method_reference_constant()?,
                11 => self.read_interface_method_reference_constant()?,
                12 => self.read_name_and_type_constant()?,
                _ => {
                    warn!("found invalid constant at index {} of type {}", i, tag);
                    return Err(ClassReaderError::InvalidClassData(format!(
                        "Unknown constant type: 0x{:X}",
                        tag
                    )));
                }
            };
            self.class_file.constants.add(constant);

            i += 1;
        }

        Ok(())
    }

    fn read_utf8_constant(&mut self) -> Result<ConstantPoolEntry> {
        let len = self.buffer.read_u16()?;
        self.buffer
            .read_utf8(len as usize)
            .map(ConstantPoolEntry::Utf8)
    }

    fn read_int_constant(&mut self) -> Result<ConstantPoolEntry> {
        self.buffer.read_i32().map(ConstantPoolEntry::Integer)
    }

    fn read_float_constant(&mut self) -> Result<ConstantPoolEntry> {
        self.buffer.read_f32().map(ConstantPoolEntry::Float)
    }

    fn read_long_constant(&mut self) -> Result<ConstantPoolEntry> {
        self.buffer.read_i64().map(ConstantPoolEntry::Long)
    }

    fn read_double_constant(&mut self) -> Result<ConstantPoolEntry> {
        self.buffer.read_f64().map(ConstantPoolEntry::Double)
    }

    fn read_class_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let fqn_string_index = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::ClassReference(fqn_string_index))
    }

    fn read_string_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let string_index = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::StringReference(string_index))
    }

    fn read_method_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let class_reference = self.buffer.read_u16()?;
        let name_and_type = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::MethodReference(
            class_reference,
            name_and_type,
        ))
    }

    fn read_interface_method_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let class_reference = self.buffer.read_u16()?;
        let name_and_type = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::InterfaceMethodReference(
            class_reference,
            name_and_type,
        ))
    }

    fn read_field_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let class_reference = self.buffer.read_u16()?;
        let name_and_type = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::FieldReference(
            class_reference,
            name_and_type,
        ))
    }

    fn read_name_and_type_constant(&mut self) -> Result<ConstantPoolEntry> {
        let name = self.buffer.read_u16()?;
        let type_descriptor = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::NameAndTypeDescriptor(
            name,
            type_descriptor,
        ))
    }

    fn read_access_flags(&mut self) -> Result<()> {
        let num = self.buffer.read_u16()?;
        match ClassAccessFlags::from_bits(num) {
            Some(flags) => {
                self.class_file.flags = flags;
                Ok(())
            }
            None => Err(ClassReaderError::InvalidClassData(format!(
                "invalid class flags: {}",
                num
            ))),
        }
    }

    fn read_class_reference(&mut self) -> Result<String> {
        let super_constant_idx = self.buffer.read_u16()?;
        if super_constant_idx == 0 {
            Ok(String::from(""))
        } else {
            self.read_string_reference(super_constant_idx)
        }
    }

    fn read_string_reference(&self, index: u16) -> Result<String> {
        self.class_file
            .constants
            .text_of(index)
            .map_err(|err| err.into())
    }

    fn read_interfaces(&mut self) -> Result<()> {
        let interfaces_count = self.buffer.read_u16()?;
        self.class_file.interfaces = (0..interfaces_count)
            .map(|_| self.read_class_reference())
            .collect::<Result<Vec<String>>>()?;
        Ok(())
    }

    fn read_fields(&mut self) -> Result<()> {
        let fields_count = self.buffer.read_u16()?;
        self.class_file.fields = (0..fields_count)
            .map(|_| self.read_field())
            .collect::<Result<Vec<ClassFileField>>>()?;
        Ok(())
    }

    fn read_field(&mut self) -> Result<ClassFileField> {
        let flags = self.read_field_flags()?;
        let name_constant_index = self.buffer.read_u16()?;
        let name = self.read_string_reference(name_constant_index)?;
        let type_constant_index = self.buffer.read_u16()?;
        let type_descriptor = self.read_string_reference(type_constant_index)?;

        let raw_attributes = self.read_raw_attributes()?;
        let constant_value = self.extract_constant_value(raw_attributes)?;

        Ok(ClassFileField {
            flags,
            name,
            type_descriptor,
            constant_value,
        })
    }

    fn read_field_flags(&mut self) -> Result<FieldFlags> {
        let field_flags_bits = self.buffer.read_u16()?;
        match FieldFlags::from_bits(field_flags_bits) {
            Some(flags) => Ok(flags),
            None => Err(ClassReaderError::InvalidClassData(format!(
                "invalid field flags: {}",
                field_flags_bits
            ))),
        }
    }

    fn extract_constant_value(
        &self,
        raw_attributes: Vec<Attribute>,
    ) -> Result<Option<FieldConstantValue>> {
        raw_attributes
            .iter()
            .filter(|attr| attr.name == "ConstantValue")
            .map(|attr| {
                if attr.info.len() != std::mem::size_of::<u16>() {
                    Err(InvalidClassData(
                        "invalid attribute of type ConstantValue".to_string(),
                    ))
                } else {
                    let attribute_bytes: &[u8] = &attr.info;
                    let constant_index = u16::from_be_bytes(attribute_bytes.try_into().unwrap());
                    self.class_file
                        .constants
                        .get(constant_index)
                        .map_err(|err| err.into())
                        .and_then(|entry| match entry {
                            ConstantPoolEntry::StringReference(v) => {
                                let referred_string = self.read_string_reference(*v)?;
                                Ok(FieldConstantValue::String(referred_string))
                            }
                            ConstantPoolEntry::Integer(v) => Ok(FieldConstantValue::Int(*v)),
                            ConstantPoolEntry::Float(v) => Ok(FieldConstantValue::Float(*v)),
                            ConstantPoolEntry::Long(v) => Ok(FieldConstantValue::Long(*v)),
                            ConstantPoolEntry::Double(v) => Ok(FieldConstantValue::Double(*v)),
                            v => Err(InvalidClassData(format!(
                                "invalid type for ConstantValue: {:?}",
                                v
                            ))),
                        })
                }
            })
            .next()
            .invert()
    }

    fn read_methods(&mut self) -> Result<()> {
        let methods_count = self.buffer.read_u16()?;
        self.class_file.methods = (0..methods_count)
            .map(|_| self.read_method())
            .collect::<Result<Vec<ClassFileMethod>>>()?;
        Ok(())
    }

    fn read_method(&mut self) -> Result<ClassFileMethod> {
        let flags = self.read_method_flags()?;
        let name_constant_index = self.buffer.read_u16()?;
        let name = self.read_string_reference(name_constant_index)?;
        let type_constant_index = self.buffer.read_u16()?;
        let type_descriptor = self.read_string_reference(type_constant_index)?;
        let attributes = self.read_raw_attributes()?;

        Ok(ClassFileMethod {
            flags,
            name,
            type_descriptor,
            attributes,
        })
    }

    fn read_method_flags(&mut self) -> Result<MethodFlags> {
        let method_flags_bits = self.buffer.read_u16()?;
        match MethodFlags::from_bits(method_flags_bits) {
            Some(flags) => Ok(flags),
            None => Err(ClassReaderError::InvalidClassData(format!(
                "invalid method flags: {}",
                method_flags_bits
            ))),
        }
    }

    fn read_raw_attributes(&mut self) -> Result<Vec<Attribute>> {
        let attributes_count = self.buffer.read_u16()?;
        (0..attributes_count)
            .map(|_| self.read_raw_attribute())
            .collect::<Result<Vec<Attribute>>>()
    }

    fn read_raw_attribute(&mut self) -> Result<Attribute> {
        let name_constant_index = self.buffer.read_u16()?;
        let name = self.read_string_reference(name_constant_index)?;
        let len = self.buffer.read_u32()?;
        let bytes = self
            .buffer
            .read_bytes(usize::try_from(len).expect("usize should have at least 32 bits"))?;
        Ok(Attribute {
            name,
            info: Vec::from(bytes),
        })
    }
}

pub fn read(path: &Path) -> Result<ClassFile> {
    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;

    read_buffer(&buf)
}

#[tracing::instrument]
pub fn read_buffer(buf: &[u8]) -> Result<ClassFile> {
    ClassFileReader::new(buf).read()
}

#[cfg(test)]
mod tests {
    use crate::class_reader::read_buffer;
    use crate::class_reader_error::ClassReaderError;

    #[test]
    fn magic_number_is_required() {
        let data = vec![0x00, 0x01, 0x02, 0x03];
        assert!(matches!(
            read_buffer(&data),
            Err(ClassReaderError::InvalidClassData(s)) if s == "invalid magic number"
        ));
    }
}
