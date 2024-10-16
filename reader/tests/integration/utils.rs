use log::info;
use rjvm_reader::class::{file::ClassFile, reader::read_buffer};

pub fn read_class_from_bytes(bytes: &[u8]) -> ClassFile {
    let class = read_buffer(bytes).unwrap();
    info!("read class file: {}", class);
    class
}
