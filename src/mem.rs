use std::{
    io::{Cursor, Read, Seek, Write},
    mem::MaybeUninit,
};

#[derive(Clone, Copy)]
pub struct Byte1 {
    bp: u8,
}

impl Byte1 {
    pub fn new(bp: u8) -> Self {
        Self { bp }
    }

    pub fn word(&self) -> bool {
        self.bp & 0b1 > 0
    }

    pub fn reg_is_dest(&self) -> bool {
        self.bp & 0b10 > 0
    }

    pub fn opcode(&self) -> u8 {
        self.bp >> 2
    }

    pub fn to_u8(&self) -> u8 {
        self.bp
    }
}

pub struct Byte2 {
    bp: u8,
}

impl Byte2 {
    pub fn new(bp: u8) -> Self {
        Self { bp }
    }

    pub fn to_u8(&self) -> u8 {
        self.bp
    }

    pub fn modd(&self) -> u8 {
        self.bp >> 6
    }

    pub fn rm(&self) -> u8 {
        self.bp & 0b111
    }

    pub fn reg(&self) -> u8 {
        (self.bp >> 3) & 0b111
    }
}

pub struct Mem {
    cursor: Cursor<Vec<u8>>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::new(Vec::with_capacity(1024 * 1024)),
        }
    }

    pub fn read_u8(&mut self) -> u8 {
        let mut buf = [0u8];
        self.cursor.read_exact(&mut buf).expect("failed to read u8");
        buf[0]
    }

    pub fn read_u16(&mut self) -> u16 {
        let mut buf = [0u8, 0];
        self.cursor
            .read_exact(&mut buf)
            .expect("failed to read u16");
        u16::from_le_bytes(buf)
    }

    pub fn read_i8(&mut self) -> i8 {
        let mut buf = [0u8];
        self.cursor
            .read_exact(&mut buf)
            .expect("failed to read i16");
        i8::from_le_bytes(buf)
    }

    pub fn read_i16(&mut self) -> i16 {
        let mut buf = [0u8, 0];
        self.cursor
            .read_exact(&mut buf)
            .expect("failed to read i16");
        i16::from_le_bytes(buf)
    }

    pub fn write_u8(&mut self, val: u8) {
        self.cursor
            .write_all(&val.to_le_bytes())
            .expect("failed to write u8");
        self.cursor.flush();
    }

    pub fn write_u16(&mut self, val: u16) {
        self.cursor
            .write_all(&val.to_le_bytes())
            .expect("failed to write u16");
        self.cursor.flush();
    }

    pub fn write_i8(&mut self, val: u8) {
        self.cursor
            .write_all(&val.to_le_bytes())
            .expect("failed to r i8");
        self.cursor.flush();
    }

    pub fn write_i16(&mut self, val: i16) {
        self.cursor
            .write_all(&val.to_le_bytes())
            .expect("failed to read i16");
        self.cursor.flush();
    }

    pub fn seek_to(&mut self, val: u64) {
        self.cursor.set_position(val);
    }

    pub fn seek_by(&mut self, val: i64) {
        self.cursor
            .seek_relative(val)
            .expect("failed to seek thy kindom");
    }

    pub fn pos(&self) -> u64 {
        self.cursor.position()
    }
}

#[cfg(test)]
mod mem_test {
    use std::io::Write;

    use super::Mem;

    #[test]
    fn a() {
        let mut m = Mem::new();
        m.write_i8(70);
        m.seek_to(0);
        assert_eq!(m.read_i8(), 70);
        m.seek_to(0);
        m.write_i16(6000);
        m.seek_by(-2);
        assert_eq!(m.read_i16(), 6000);
    }
}
