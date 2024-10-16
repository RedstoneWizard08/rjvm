use std::{fmt, vec::Vec};
use thiserror::Error;

/// Types of a constant in the constant pool of a class, following the JVM spec:
/// https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.4
#[derive(Debug, PartialEq)]
pub enum ConstantPoolEntry {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    ClassReference(u16),
    StringReference(u16),
    FieldReference(u16, u16),
    MethodReference(u16, u16),
    InterfaceMethodReference(u16, u16),
    NameAndTypeDescriptor(u16, u16),
    MethodHandle(u8, u16),
    MethodType(u16),
    DynamicInfo(u16, u16),
    InvokeDynamicInfo(u16, u16),
    ModuleInfo(u16),
    PackageInfo(u16),
}

/// Constants in the pool generally take one slot, but long and double take two. We do not use
/// the second one, so we have a tombstone to ensure the indexes match.
#[derive(Debug)]
enum ConstantPoolPhysicalEntry {
    Entry(ConstantPoolEntry),
    MultiByteEntryTombstone(),
}

/// Implementation of the constant pool of a java class.
/// Note that constants are 1-based in java.
#[derive(Debug, Default)]
pub struct ConstantPool {
    entries: Vec<ConstantPoolPhysicalEntry>,
}

/// Error used to signal that an attempt was made to access a non existing constant pool entry.
#[derive(Error, Debug, PartialEq, Eq)]
#[error("invalid constant pool index: {index}")]
pub struct InvalidConstantPoolIndexError {
    pub index: u16,
}

#[derive(Error, Debug, PartialEq, Eq)]
#[error("invalid method handle kind: {kind}")]
pub struct InvalidMethodHandleKindError {
    pub kind: u8,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConstantPoolFormattingError {
    #[error(transparent)]
    PoolIndex(#[from] InvalidConstantPoolIndexError),

    #[error(transparent)]
    MethodHandleKind(#[from] InvalidMethodHandleKindError),
}

impl InvalidConstantPoolIndexError {
    fn new(index: u16) -> Self {
        InvalidConstantPoolIndexError { index }
    }
}

impl ConstantPool {
    pub fn new() -> ConstantPool {
        Default::default()
    }

    /// Adds a new entry.
    pub fn add(&mut self, entry: ConstantPoolEntry) {
        let add_tombstone = matches!(
            &entry,
            ConstantPoolEntry::Long(_) | ConstantPoolEntry::Double(_)
        );
        self.entries.push(ConstantPoolPhysicalEntry::Entry(entry));

        if add_tombstone {
            self.entries
                .push(ConstantPoolPhysicalEntry::MultiByteEntryTombstone())
        }
    }

    /// Accesses an entry given its index. Note that it must be 1-based!
    pub fn get(
        &self,
        input_index: u16,
    ) -> Result<&ConstantPoolEntry, InvalidConstantPoolIndexError> {
        if input_index == 0 || input_index as usize > self.entries.len() {
            Err(InvalidConstantPoolIndexError::new(input_index))
        } else {
            let i = (input_index - 1) as usize;
            let entry = &self.entries[i];
            match entry {
                ConstantPoolPhysicalEntry::Entry(entry) => Ok(entry),
                ConstantPoolPhysicalEntry::MultiByteEntryTombstone() => {
                    Err(InvalidConstantPoolIndexError::new(input_index))
                }
            }
        }
    }

    fn fmt_entry(&self, idx: u16) -> Result<String, ConstantPoolFormattingError> {
        let entry = self.get(idx)?;
        let text = match entry {
            ConstantPoolEntry::Utf8(ref s) => format!("String: \"{s}\""),
            ConstantPoolEntry::Integer(n) => format!("Integer: {n}"),
            ConstantPoolEntry::Float(n) => format!("Float: {n}"),
            ConstantPoolEntry::Long(n) => format!("Long: {n}"),
            ConstantPoolEntry::Double(n) => format!("Double: {n}"),
            ConstantPoolEntry::ClassReference(n) => {
                format!("ClassReference: {} => ({})", n, self.fmt_entry(*n)?)
            }
            ConstantPoolEntry::StringReference(n) => {
                format!("StringReference: {} => ({})", n, self.fmt_entry(*n)?)
            }
            ConstantPoolEntry::FieldReference(i, j) => {
                format!(
                    "FieldReference: {}, {} => ({}), ({})",
                    i,
                    j,
                    self.fmt_entry(*i)?,
                    self.fmt_entry(*j)?
                )
            }
            ConstantPoolEntry::MethodReference(i, j) => {
                format!(
                    "MethodReference: {}, {} => ({}), ({})",
                    i,
                    j,
                    self.fmt_entry(*i)?,
                    self.fmt_entry(*j)?
                )
            }
            ConstantPoolEntry::InterfaceMethodReference(i, j) => {
                format!(
                    "InterfaceMethodReference: {}, {} => ({}), ({})",
                    i,
                    j,
                    self.fmt_entry(*i)?,
                    self.fmt_entry(*j)?
                )
            }
            &ConstantPoolEntry::NameAndTypeDescriptor(i, j) => {
                format!(
                    "NameAndTypeDescriptor: {}, {} => ({}), ({})",
                    i,
                    j,
                    self.fmt_entry(i)?,
                    self.fmt_entry(j)?
                )
            }
            &ConstantPoolEntry::MethodHandle(i, j) => {
                format!(
                    "MethodHandle: {}, {} => ({}), ({})",
                    i,
                    j,
                    self.method_handle_kind(i)?,
                    self.fmt_entry(j)?
                )
            }
            &ConstantPoolEntry::MethodType(i) => {
                format!("MethodType: {} => ({})", i, self.fmt_entry(i)?)
            }
            &ConstantPoolEntry::DynamicInfo(i, j) => {
                format!(
                    "DynamicInfo: {}, {} => ({}), ({})",
                    i,
                    j,
                    self.fmt_entry(i)?,
                    self.fmt_entry(j)?
                )
            }
            &ConstantPoolEntry::InvokeDynamicInfo(i, j) => {
                format!(
                    "InvokeDynamicInfo: {}, {} => ({}), ({})",
                    i,
                    j,
                    self.fmt_entry(i)?,
                    self.fmt_entry(j)?
                )
            }
            &ConstantPoolEntry::ModuleInfo(i) => {
                format!("ModuleInfo: {} => ({})", i, self.fmt_entry(i)?)
            }
            &ConstantPoolEntry::PackageInfo(i) => {
                format!("PackageInfo: {} => ({})", i, self.fmt_entry(i)?)
            }
        };
        Ok(text)
    }

    pub fn text_of(&self, idx: u16) -> Result<String, ConstantPoolFormattingError> {
        let entry = self.get(idx)?;
        let text = match entry {
            ConstantPoolEntry::Utf8(ref s) => s.clone(),
            ConstantPoolEntry::Integer(n) => n.to_string(),
            ConstantPoolEntry::Float(n) => n.to_string(),
            ConstantPoolEntry::Long(n) => n.to_string(),
            ConstantPoolEntry::Double(n) => n.to_string(),
            ConstantPoolEntry::ClassReference(n) => self.text_of(*n)?,
            ConstantPoolEntry::StringReference(n) => self.text_of(*n)?,
            ConstantPoolEntry::FieldReference(i, j) => {
                format!("{}.{}", self.text_of(*i)?, self.text_of(*j)?)
            }
            ConstantPoolEntry::MethodReference(i, j) => {
                format!("{}.{}", self.text_of(*i)?, self.text_of(*j)?)
            }
            ConstantPoolEntry::InterfaceMethodReference(i, j) => {
                format!("{}.{}", self.text_of(*i)?, self.text_of(*j)?)
            }
            ConstantPoolEntry::NameAndTypeDescriptor(i, j) => {
                format!("{}: {}", self.text_of(*i)?, self.text_of(*j)?)
            }
            ConstantPoolEntry::MethodHandle(i, j) => {
                format!("{}({})", self.method_handle_kind(*i)?, self.text_of(*j)?)
            }
            ConstantPoolEntry::MethodType(i) => self.text_of(*i)?,
            ConstantPoolEntry::DynamicInfo(i, j) => {
                format!("{}: {}", self.text_of(*i)?, self.text_of(*j)?)
            }
            ConstantPoolEntry::InvokeDynamicInfo(i, j) => {
                format!("{}: {}", self.text_of(*i)?, self.text_of(*j)?)
            }
            ConstantPoolEntry::ModuleInfo(i) => self.text_of(*i)?,
            ConstantPoolEntry::PackageInfo(i) => self.text_of(*i)?,
        };
        Ok(text)
    }

    pub fn method_handle_kind(&self, kind: u8) -> Result<String, InvalidMethodHandleKindError> {
        Ok(match kind {
            1 => "getField",
            2 => "getStatic",
            3 => "putField",
            4 => "putStatic",
            5 => "invokeVirtual",
            6 => "invokeStatic",
            7 => "invokeSpecial",
            8 => "newInvokeSpecial",
            9 => "invokeInterface",
            _ => return Err(InvalidMethodHandleKindError { kind }),
        }
        .into())
    }
}

impl fmt::Display for ConstantPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Constant pool: (size: {})", self.entries.len())?;
        for (raw_idx, _) in self.entries.iter().enumerate() {
            let index = (raw_idx + 1) as u16;
            let entry_text = self.fmt_entry(index).map_err(|_| fmt::Error::default())?;
            writeln!(f, "    {}, {}", index, entry_text)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        constant_pool::{ConstantPool, ConstantPoolEntry, InvalidConstantPoolIndexError},
        ConstantPoolFormattingError,
    };

    #[test]
    fn constant_pool_works() {
        let mut cp = ConstantPool::new();
        cp.add(ConstantPoolEntry::Utf8("hey".to_string()));
        cp.add(ConstantPoolEntry::Integer(1));
        cp.add(ConstantPoolEntry::Float(2.1));
        cp.add(ConstantPoolEntry::Long(123));
        cp.add(ConstantPoolEntry::Double(3.56));
        cp.add(ConstantPoolEntry::ClassReference(1));
        cp.add(ConstantPoolEntry::StringReference(1));
        cp.add(ConstantPoolEntry::Utf8("joe".to_string()));
        cp.add(ConstantPoolEntry::FieldReference(1, 10));
        cp.add(ConstantPoolEntry::MethodReference(1, 10));
        cp.add(ConstantPoolEntry::InterfaceMethodReference(1, 10));
        cp.add(ConstantPoolEntry::NameAndTypeDescriptor(1, 10));

        assert_eq!(
            ConstantPoolEntry::Utf8("hey".to_string()),
            *cp.get(1).unwrap()
        );
        assert_eq!(ConstantPoolEntry::Integer(1), *cp.get(2).unwrap());
        assert_eq!(ConstantPoolEntry::Float(2.1), *cp.get(3).unwrap());
        assert_eq!(ConstantPoolEntry::Long(123i64), *cp.get(4).unwrap());
        assert_eq!(Err(InvalidConstantPoolIndexError::new(5)), cp.get(5));
        assert_eq!(ConstantPoolEntry::Double(3.56), *cp.get(6).unwrap());
        assert_eq!(Err(InvalidConstantPoolIndexError::new(7)), cp.get(7));
        assert_eq!(ConstantPoolEntry::ClassReference(1), *cp.get(8).unwrap());
        assert_eq!(ConstantPoolEntry::StringReference(1), *cp.get(9).unwrap());
        assert_eq!(
            ConstantPoolEntry::Utf8("joe".to_string()),
            *cp.get(10).unwrap()
        );
        assert_eq!(
            ConstantPoolEntry::FieldReference(1, 10),
            *cp.get(11).unwrap()
        );
        assert_eq!(
            ConstantPoolEntry::MethodReference(1, 10),
            *cp.get(12).unwrap()
        );
        assert_eq!(
            ConstantPoolEntry::InterfaceMethodReference(1, 10),
            *cp.get(13).unwrap()
        );
        assert_eq!(
            ConstantPoolEntry::NameAndTypeDescriptor(1, 10),
            *cp.get(14).unwrap()
        );

        assert_eq!("hey", cp.text_of(1).unwrap());
        assert_eq!("1", cp.text_of(2).unwrap());
        assert_eq!("2.1", cp.text_of(3).unwrap());
        assert_eq!("123", cp.text_of(4).unwrap());
        assert_eq!(
            Err(ConstantPoolFormattingError::PoolIndex(
                InvalidConstantPoolIndexError::new(5)
            )),
            cp.text_of(5)
        );
        assert_eq!("3.56", cp.text_of(6).unwrap());
        assert_eq!(
            Err(ConstantPoolFormattingError::PoolIndex(
                InvalidConstantPoolIndexError::new(7)
            )),
            cp.text_of(7)
        );
        assert_eq!("hey", cp.text_of(8).unwrap());
        assert_eq!("hey", cp.text_of(9).unwrap());
        assert_eq!("joe", cp.text_of(10).unwrap());
        assert_eq!("hey.joe", cp.text_of(11).unwrap());
        assert_eq!("hey.joe", cp.text_of(12).unwrap());
        assert_eq!("hey.joe", cp.text_of(13).unwrap());
        assert_eq!("hey: joe", cp.text_of(14).unwrap());
    }
}
