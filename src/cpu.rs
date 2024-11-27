use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
    ops::{Add, Deref},
    u8::{self, MAX},
};

use crate::{
    mem::{Byte1, Byte2, Mem},
    regs::Registers,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operand {
    Mem16(u32),
    Mem8(u32),
    Reg8(u8),
    Reg16(u8),
    Imm8(u8),
    Imm16(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Opcode {
    Add,
    PushEs,
    PopEs,
}

#[derive(Debug)]
pub struct Instruction {
    opcode: Opcode,
    dest: Operand,
    src: Operand,
}

pub enum Segment {
    Ds,
    Es,
    Ss,
    Cs,
}

impl Instruction {
    pub fn opcode(&self) -> Opcode {
        self.opcode
    }

    pub fn operands(&self) -> (Operand, Operand) {
        (self.dest, self.src)
    }
}

pub struct Cpu {
    pub regs: Registers,
    pub mem: Mem,
    pub prog_size: u64,
    pub seg_override: Option<Segment>,
}

impl Cpu {
    pub fn init() -> Self {
        let mut cpu = Self {
            prog_size: 0,
            regs: Registers::default(),
            mem: Mem::new(),
            seg_override: None,
        };
        cpu.regs.set_cs(0);
        cpu.regs.set_ds(4096);
        cpu.regs.set_ss(8192);
        cpu.regs.set_es(8192 + 4096);
        cpu
    }

    pub fn get_seg_reg(&self, pos: u8) -> u32 {
        match pos & 0b11 {
            0 => self.regs.get_es(),
            1 => self.regs.get_cs(),
            2 => self.regs.get_ss(),
            3 => self.regs.get_ds(),
            4u8..=u8::MAX => panic!("invalid seg reg {}", pos),
        }
    }

    pub fn set_seg_reg(&mut self, pos: u8, val: u32) {
        match pos & 0b11 {
            0 => self.regs.set_es(val),
            1 => self.regs.set_cs(val),
            2 => self.regs.set_ss(val),
            3 => self.regs.set_ds(val),
            4u8..=u8::MAX => panic!("invalid seg reg {}", pos),
        };
    }

    pub fn get_reg(&self, id: u8, word: bool) -> u16 {
        if word {
            match id {
                0 => self.regs.get_ax(),
                3 => self.regs.get_bx(),
                2 => self.regs.get_dx(),
                6 => self.regs.get_si(),
                4 => self.regs.get_sp(),
                5 => self.regs.get_bp(),
                7 => self.regs.get_di(),
                1 => self.regs.get_cx(),
                8..=u8::MAX => panic!("invalid register: {}", id),
            }
        } else {
            (match id {
                0 => self.regs.get_al(),
                3 => self.regs.get_bl(),
                2 => self.regs.get_dl(),
                6 => self.regs.get_dh(),
                4 => self.regs.get_ah(),
                5 => self.regs.get_ch(),
                7 => self.regs.get_bh(),
                1 => self.regs.get_cl(),
                8..=u8::MAX => panic!("invalid register: {}", id),
            }) as u16
        }
    }

    pub fn set_reg(&mut self, id: u8, word: bool, val: u16) {
        if word {
            match id {
                0 => self.regs.set_ax(val as u16),
                3 => self.regs.set_bx(val as u16),
                2 => self.regs.set_dx(val as u16),
                6 => self.regs.set_si(val as u16),
                4 => self.regs.set_sp(val as u16),
                5 => self.regs.set_bp(val as u16),
                7 => self.regs.set_di(val as u16),
                1 => self.regs.set_cx(val as u16),
                8..=u8::MAX => panic!("invalid register: {}", id),
            };
        } else {
            (match id {
                0 => self.regs.set_al(val as u8),
                3 => self.regs.set_bl(val as u8),
                2 => self.regs.set_dl(val as u8),
                6 => self.regs.set_dh(val as u8),
                4 => self.regs.set_ah(val as u8),
                5 => self.regs.set_ch(val as u8),
                7 => self.regs.set_bh(val as u8),
                1 => self.regs.set_cl(val as u8),
                8..=u8::MAX => panic!("invalid register: {}", id),
            });
        };
    }

    pub fn ea(&self, seg: &Segment, offt: u32) -> u32 {
        match seg {
            Segment::Ds => self.regs.get_ds() + offt,
            Segment::Es => self.regs.get_es() + offt,
            Segment::Ss => self.regs.get_ss() + offt,
            Segment::Cs => self.regs.get_cs() + offt,
        }
    }

    pub fn get_segment_offset(&mut self, seg: Segment, offt: u32) -> u32 {
        if let Some(ov) = &self.seg_override {
            let res = self.ea(ov, offt);
            self.seg_override = None;
            res
        } else {
            self.ea(&seg, offt)
        }
    }

    pub fn calc_op_displacement(&mut self, b1: Byte1, b2: Byte2) -> Operand {
        match b2.modd() {
            0 => match b2.rm() {
                0 => {
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(
                            Segment::Ds,
                            (self.regs.get_bx() + self.regs.get_si()) as u32,
                        ))
                    } else {
                        Operand::Mem8(self.get_segment_offset(
                            Segment::Ds,
                            (self.regs.get_bx() + self.regs.get_si()) as u32,
                        ))
                    }
                }
                1 => {
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(
                            Segment::Ds,
                            (self.regs.get_bx() + self.regs.get_di()) as u32,
                        ))
                    } else {
                        Operand::Mem8(self.get_segment_offset(
                            Segment::Ds,
                            (self.regs.get_bx() + self.regs.get_di()) as u32,
                        ))
                    }
                }
                2 => {
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(
                            Segment::Ss,
                            (self.regs.get_bp() + self.regs.get_si()) as u32,
                        ))
                    } else {
                        Operand::Mem8(self.get_segment_offset(
                            Segment::Ss,
                            (self.regs.get_bp() + self.regs.get_si()) as u32,
                        ))
                    }
                }
                3 => {
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(
                            Segment::Ss,
                            (self.regs.get_bp() + self.regs.get_di()) as u32,
                        ))
                    } else {
                        Operand::Mem8(self.get_segment_offset(
                            Segment::Ss,
                            (self.regs.get_bp() + self.regs.get_di()) as u32,
                        ))
                    }
                }
                4 => {
                    if b1.word() {
                        Operand::Mem16(
                            self.get_segment_offset(Segment::Ds, (self.regs.get_si()) as u32),
                        )
                    } else {
                        Operand::Mem8(
                            self.get_segment_offset(Segment::Ds, (self.regs.get_si()) as u32),
                        )
                    }
                }
                5 => {
                    if b1.word() {
                        Operand::Mem16(
                            self.get_segment_offset(Segment::Ds, (self.regs.get_di()) as u32),
                        )
                    } else {
                        Operand::Mem8(
                            self.get_segment_offset(Segment::Ds, (self.regs.get_di()) as u32),
                        )
                    }
                }
                6 => {
                    let offt = self.mem.read_u16() as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ds, offt))
                    } else {
                        Operand::Mem8(self.get_segment_offset(Segment::Ds, offt))
                    }
                }
                7 => {
                    if b1.word() {
                        Operand::Mem16(
                            self.get_segment_offset(Segment::Ds, (self.regs.get_bx()) as u32),
                        )
                    } else {
                        Operand::Mem8(
                            self.get_segment_offset(Segment::Ds, (self.regs.get_bx()) as u32),
                        )
                    }
                }
                8..=u8::MAX => unreachable!(),
            },
            0b1 => {
                let disp = self.mem.read_u8() as u16;
                let res = match b2.rm() {
                    0 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + self.regs.get_si() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + self.regs.get_si() + disp) as u32,
                            ))
                        }
                    }
                    1 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + self.regs.get_di() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + self.regs.get_di() + disp) as u32,
                            ))
                        }
                    }
                    2 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + self.regs.get_si() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + self.regs.get_si() + disp) as u32,
                            ))
                        }
                    }
                    3 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + self.regs.get_di() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + self.regs.get_di() + disp) as u32,
                            ))
                        }
                    }
                    4 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_si() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_si() + disp) as u32,
                            ))
                        }
                    }
                    5 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_di() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_di() + disp) as u32,
                            ))
                        }
                    }
                    6 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + disp) as u32,
                            ))
                        }
                    }
                    7 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + disp) as u32,
                            ))
                        }
                    }
                    8..=u8::MAX => unreachable!(),
                };
                res
            }
            0b10 => {
                let disp = self.mem.read_u16();
                let res = match b2.rm() {
                    0 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + self.regs.get_si() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + self.regs.get_si() + disp) as u32,
                            ))
                        }
                    }
                    1 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + self.regs.get_di() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + self.regs.get_di() + disp) as u32,
                            ))
                        }
                    }
                    2 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + self.regs.get_si() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + self.regs.get_si() + disp) as u32,
                            ))
                        }
                    }
                    3 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + self.regs.get_di() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ss,
                                (self.regs.get_bp() + self.regs.get_di() + disp) as u32,
                            ))
                        }
                    }
                    4 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_si() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_si() + disp) as u32,
                            ))
                        }
                    }
                    5 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_di() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_di() + disp) as u32,
                            ))
                        }
                    }
                    6 => {
                        let offt = self.mem.read_u16() as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt))
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt))
                        }
                    }
                    7 => {
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + disp) as u32,
                            ))
                        } else {
                            Operand::Mem8(self.get_segment_offset(
                                Segment::Ds,
                                (self.regs.get_bx() + disp) as u32,
                            ))
                        }
                    }
                    8..=u8::MAX => unreachable!(),
                };
                res
            }
            0b11..=u8::MAX => unreachable!(),
        }
    }

    pub fn fetch(&mut self) -> Option<Instruction> {
        if self.mem.pos() >= self.prog_size {
            return None;
        }

        let mut result = (Operand::Mem16(0), Operand::Mem16(0));
        let b1 = Byte1::new(self.mem.read_u8());

        match b1.opcode() {
            0 => {
                let b2 = Byte2::new(self.mem.read_u8());

                if (b1.reg_is_dest()) {
                    result.0 = match b1.word() {
                        true => Operand::Reg16(b2.reg()),
                        false => Operand::Reg8(b2.reg()),
                    };

                    result.1 = match b2.modd() {
                        3 => match b1.word() {
                            true => Operand::Reg16(b2.rm()),
                            false => Operand::Reg8(b2.rm()),
                        },
                        _ => self.calc_op_displacement(b1, b2),
                    }
                } else {
                    result.1 = match b1.word() {
                        true => Operand::Reg16(b2.reg()),
                        false => Operand::Reg8(b2.reg()),
                    };

                    result.0 = match b2.modd() {
                        3 => match b1.word() {
                            true => Operand::Reg16(b2.rm()),
                            false => Operand::Reg8(b2.rm()),
                        },
                        _ => self.calc_op_displacement(b1, b2),
                    };
                }

                Some(Instruction {
                    opcode: Opcode::Add,
                    dest: result.0,
                    src: result.1,
                })
            }
            1 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Add,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Add,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::PushEs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::PopEs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn operand_value(&mut self, op: Operand) -> u16 {
        let pos = self.mem.pos();
        let val = match op {
            Operand::Mem16(i) => {
                self.mem.seek_to(i as u64);
                self.mem.read_u16()
            }
            Operand::Mem8(i) => {
                self.mem.seek_to(i as u64);
                self.mem.read_u8() as u16
            }
            Operand::Reg8(i) => self.get_reg(i, false),
            Operand::Reg16(i) => self.get_reg(i, true),
            Operand::Imm8(i) => i as u16,
            Operand::Imm16(i) => i,
        };
        self.mem.seek_to(pos);
        val
    }

    fn write_mem_u16(&mut self, pos: u32, val: u16) {
        let p = self.mem.pos();
        self.mem.seek_to(pos as u64);
        self.mem.write_u16(val);
        self.mem.seek_to(p);
    }

    fn write_mem_u8(&mut self, pos: u32, val: u8) {
        let p = self.mem.pos();
        self.mem.seek_to(pos as u64);
        self.mem.write_u8(val);
        self.mem.seek_to(p);
    }

    fn read_mem_u16(&mut self, pos: u32) -> u16 {
        let p = self.mem.pos();
        self.mem.seek_to(pos as u64);
        let res = self.mem.read_u16();
        self.mem.seek_to(p);
        res
    }

    fn read_mem_u8(&mut self, pos: u32, val: u8) -> u8 {
        let p = self.mem.pos();
        self.mem.seek_to(pos as u64);
        let res = self.mem.read_u8();
        self.mem.seek_to(p);
        res
    }

    //pub fn add()

    fn even_parity(mut val: u8) -> bool {
        let mut res = 0;
        while val > 0 {
            res += val & 1;
            val >>= 1;
        }
        res % 2 == 0
    }

    //fn will_ovf8(a: u8, b: u8) -> bool {
    //    let res =  add_with_overflow(a, b)
    //}

    fn aux_add(a: u16, b: u16) -> bool {
        (a & 0b1111) + (b & 0b1111) > 0b1111
    }

    // a - b
    fn aux_sub(a: u16, b: u16) -> bool {
        (a & 0b1111) < (b & 0b1111)
    }

    fn add(&mut self, d: Operand, s: Operand) {
        self.regs.flags.clear_arith();

        let dest = self.operand_value(d);
        let src = self.operand_value(s);

        let result = dest.wrapping_add(src);

        if (Self::aux_add(dest, src)) {
            self.regs.flags.set_af();
        }

        if Self::even_parity(result as u8) {
            self.regs.flags.set_pf();
        }

        if result == 0 {
            self.regs.flags.set_zf();
        }

        match d {
            Operand::Mem16(p) => {
                if (dest as i16).overflowing_add(src as i16).1 {
                    self.regs.flags.set_of();
                }

                if (dest as u16).overflowing_add(src as u16).1 {
                    self.regs.flags.set_cf();
                }

                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.write_mem_u16(p, result)
            }
            Operand::Mem8(p) => {
                if (dest as i8).overflowing_add(src as i8).1 {
                    self.regs.flags.set_of();
                }

                if (dest as u8).overflowing_add(src as u8).1 {
                    self.regs.flags.set_cf();
                }

                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.write_mem_u8(p, result as u8)
            }
            Operand::Reg8(r) => {
                if (dest as i8).overflowing_add(src as i8).1 {
                    self.regs.flags.set_of();
                }

                if (dest as u8).overflowing_add(src as u8).1 {
                    self.regs.flags.set_cf();
                }

                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }
                self.set_reg(r, false, result)
            }
            Operand::Reg16(r) => {
                if (dest as i16).overflowing_add(src as i16).1 {
                    self.regs.flags.set_of();
                }

                if (dest as u16).overflowing_add(src as u16).1 {
                    self.regs.flags.set_cf();
                }

                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }
                self.set_reg(r, true, result)
            }
            _ => unreachable!("Immediate destination"),
        }
    }

    pub fn execute(&mut self, inst: &Instruction) {
        match inst.opcode {
            Opcode::Add => self.add(inst.dest, inst.src),
            Opcode::PushEs => {
                self.regs.sp.wrapping_sub(2);
                self.write_mem_u16(self.stack_addr(self.regs.sp), self.regs.es as u16);
            }
            Opcode::PopEs => {
                let es = self.read_mem_u16(self.stack_addr(self.regs.sp));
                self.regs.sp.wrapping_add(2);
                self.regs.es = es as u32;
            }
            _ => unimplemented!(),
        }
    }

    // program will be cut
    pub fn load_code(&mut self, path: &str) {
        let mut file = File::open(path).expect("failed to open bin stuff");
        self.mem.seek_to(self.code_addr(0) as u64);
        while self.mem.pos() < 1024 {
            let mut buf = [0u8];
            if let Ok(0) = file.read(&mut buf) {
                break;
            }
            self.mem.write_u8(buf[0]);
        }
        self.prog_size = self.mem.pos();
    }

    pub fn load_code_vec(&mut self, vec: &[u8]) {
        self.mem.seek_to(self.code_addr(0) as u64);
        let mut it = vec.iter();
        while self.mem.pos() < 1024 {
            if let Some(v) = it.next() {
                self.mem.write_u8(*v);
            } else {
                break;
            }
        }
        self.prog_size = self.mem.pos();
    }

    pub fn code_addr(&self, offset: u16) -> u32 {
        ((self.regs.get_cs() + offset as u32) & 0xfffff)
    }

    pub fn stack_addr(&self, offset: u16) -> u32 {
        ((self.regs.get_ss() + offset as u32) & 0xfffff)
    }

    pub fn extra_addr(&self, offset: u16) -> u32 {
        ((self.regs.get_es() + offset as u32) & 0xfffff)
    }

    pub fn data_addr(&self, offset: u16) -> u32 {
        ((self.regs.get_ds() + offset as u32) & 0xfffff)
    }
}
#[cfg(test)]
mod cpu_test {

    use super::{Cpu, Instruction};
    use crate::{
        cpu::{self, Opcode, Operand},
        mem::Byte1,
    };

    #[test]
    #[should_panic]
    fn a() {
        let mut cpu = Cpu::init();
        cpu.regs.set_cs(3);
        cpu.regs.set_ds(1024 * 64);
        cpu.regs.set_ss(1024 * 128);
        cpu.regs.set_es(1024 * 196);
    }
    #[test]
    fn b() {
        let mut cpu = Cpu::init();
        cpu.regs.set_cs(0);
        cpu.regs.set_ds(1024 * 64);
        cpu.regs.set_ss(1024 * 128);
        cpu.regs.set_es(1024 * 196);

        assert!(cpu.code_addr(0) == 0);
        assert!(cpu.code_addr(5) == 5);
        assert!(cpu.code_addr(0xffff) == 0xffff);

        cpu.regs.set_cs(0xffff + 1);
        assert!(cpu.code_addr(0) as u32 == 0xffff + 1 as u32);

        cpu.regs.set_cs(0xfffff + 1);
        assert!(cpu.code_addr(0) == 0);
    }

    #[test]
    fn c() {
        let mut cpu = Cpu::init();
        cpu.load_code_vec(&[
            0x2, 0x2e, 0x50, 0x0, 0x0, 0xc0, 0x0, 0xc9, 0x0, 0xe4, 0x0, 0xdb, 0x0, 0xff, 0x0, 0xed,
            0x0, 0xc9, 0x1, 0xc0, 0x1, 0xdb, 0x1, 0xc9, 0x1, 0xd2, 0x1, 0x6, 0x5a, 0x0, 0x3, 0x4,
            0x1, 0xf6, 0x1, 0xff, 0x1, 0xed, 0x1, 0xe4, 0x1, 0x4, 0x1, 0x1d,
        ]);
        cpu.mem.seek_to(cpu.code_addr(0) as u64);

        cpu.regs.set_cs(0);
        cpu.regs.set_ds(0);
        cpu.regs.set_ss(0);
        cpu.regs.set_es(0);

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg8(5), Operand::Mem8(80)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg8(0), Operand::Reg8(0)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg8(1), Operand::Reg8(1)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg8(4), Operand::Reg8(4)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg8(3), Operand::Reg8(3)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg8(7), Operand::Reg8(7)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg8(5), Operand::Reg8(5)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg8(1), Operand::Reg8(1)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(0), Operand::Reg16(0)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(3), Operand::Reg16(3)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(1), Operand::Reg16(1)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(2), Operand::Reg16(2)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Mem16(90), Operand::Reg16(0)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(0), Operand::Mem16(0)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(6), Operand::Reg16(6)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(7), Operand::Reg16(7)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(5), Operand::Reg16(5)));

        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Reg16(4), Operand::Reg16(4)));

        cpu.regs.set_si(90);
        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Mem16(90), Operand::Reg16(0)));

        cpu.regs.set_di(90);
        let b1 = cpu.fetch().unwrap();
        assert!(b1.opcode() == Opcode::Add);
        assert_eq!(b1.operands(), (Operand::Mem16(90), Operand::Reg16(3)));
    }

    #[test]
    fn add() {
        let mut cpu = Cpu::init();
        cpu.mem.seek_to(cpu.code_addr(0) as u64);

        cpu.regs.set_cs(0);
        cpu.regs.set_ds(0);
        cpu.regs.set_ss(0);
        cpu.regs.set_es(0);

        cpu.execute(&Instruction {
            opcode: Opcode::Add,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert!(cpu.regs.flags.zf());

        cpu.regs.set_ax(255);
        cpu.execute(&Instruction {
            opcode: Opcode::Add,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert!(cpu.regs.flags.af());
        assert!(cpu.regs.flags.cf());
        assert!(!cpu.regs.flags.of());
        assert!(cpu.regs.flags.sf());

        cpu.regs.set_ax(70);
        cpu.execute(&Instruction {
            opcode: Opcode::Add,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert!(cpu.regs.flags.of());

        let a = -127i8;

        assert!(a.overflowing_add(a).1);

        cpu.regs.set_ax(a as u16);
        cpu.execute(&Instruction {
            opcode: Opcode::Add,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert!(cpu.regs.flags.of());
    }

    #[test]
    fn push_pop_es() {
        let mut cpu = Cpu::init();
        cpu.mem.seek_to(cpu.code_addr(0) as u64);
        cpu.regs.set_cs(0);
        cpu.regs.set_ds(0);
        cpu.regs.set_ss(4096);
        cpu.regs.set_es(32);
        cpu.execute(&Instruction {
            opcode: Opcode::PushEs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.read_mem_u16(cpu.stack_addr(cpu.regs.sp)), 2);
        cpu.write_mem_u16(cpu.stack_addr(cpu.regs.sp), 64);
        cpu.execute(&Instruction {
            opcode: Opcode::PopEs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.regs.es, 64);
    }
}
