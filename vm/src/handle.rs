use rjvm_reader::{ClassFileVersion, ConstantPool, ConstantPoolEntry};

use crate::vm_error::VmError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MethodHandle {
    FieldRef(u16, u16),
    MethodRef(u16, u16),
    InterfaceMethodRef(u16, u16),
}

impl MethodHandle {
    pub fn resolve(
        version: &ClassFileVersion,
        pool: &ConstantPool,
        kind: u8,
        index: u16,
    ) -> Result<MethodHandle, VmError> {
        let entry = pool.get(index).map_err(|_| VmError::ValidationException)?;

        // TODO: For kind == 5|6|7|9 check if the method name != "<init>" or "<clinit>"
        // TODO: For kind == 8 check if method name != "<init>"

        match kind {
            1 | 2 | 3 | 4 => {
                if let ConstantPoolEntry::FieldReference(i, j) = entry {
                    Ok(Self::FieldRef(*i, *j))
                } else {
                    Err(VmError::ValidationException)
                }
            }
            5 | 8 => {
                if let ConstantPoolEntry::MethodReference(i, j) = entry {
                    Ok(Self::MethodRef(*i, *j))
                } else {
                    Err(VmError::ValidationException)
                }
            }
            6 | 7 => {
                if version.major_version() < 52 {
                    if let ConstantPoolEntry::MethodReference(i, j) = entry {
                        Ok(Self::MethodRef(*i, *j))
                    } else {
                        Err(VmError::ValidationException)
                    }
                } else {
                    if let ConstantPoolEntry::MethodReference(i, j) = entry {
                        Ok(Self::MethodRef(*i, *j))
                    } else if let ConstantPoolEntry::InterfaceMethodReference(i, j) = entry {
                        Ok(Self::InterfaceMethodRef(*i, *j))
                    } else {
                        Err(VmError::ValidationException)
                    }
                }
            }
            9 => {
                if let ConstantPoolEntry::InterfaceMethodReference(i, j) = entry {
                    Ok(Self::InterfaceMethodRef(*i, *j))
                } else {
                    Err(VmError::ValidationException)
                }
            }
            _ => Err(VmError::ValidationException),
        }
    }
}
