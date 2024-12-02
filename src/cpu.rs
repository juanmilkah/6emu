use std::{
    fs::File, io::{BufReader, Cursor, Read, Seek, SeekFrom, Stdin}, ops::{Add, Deref}, process::exit, u8::{self, MAX}
};

use crate::{
    mem::{Byte1, Byte2, Mem},
    regs::Registers,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operand {
    Mem16(u32, u32),
    Mem8(u32, u32),
    Reg8(u8),
    Reg16(u8),
    Imm8(u8),
    Imm16(u16),
    Seg(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Opcode {
    Add,
    PushEs,
    PopEs,
    Or,
    PushCs,
    Adc,
    PushSs,
    PopSs,
    Sbb,
    Sub,
    Cmp,
    PushDs,
    PopDs,
    And,
    Xor,
    OverrideEs,
    OverrideCs,
    OverrideSs,
    OverrideDs,
    Daa,
    Aas,
    Das,
    Aaa,
    IncAx,
    IncCx,
    IncBx,
    IncDx,
    IncSp,
    IncBp,
    IncSi,
    IncDi,
    DecAx,
    DecCx,
    DecBx,
    DecDx,
    DecSp,
    DecBp,
    DecSi,
    DecDi,
    PushAx,
    PushCx,
    PushBx,
    PushDx,
    PushSp,
    PushBp,
    PushSi,
    PushDi,
    PopAx,
    PopCx,
    PopBx,
    PopDx,
    PopSp,
    PopBp,
    PopSi,
    PopDi,
    Jo,
    Jno,
    Jb,
    Jnb,
    Jz,
    Jnz,
    Jbe,
    Jnbe,
    Js,
    Jns,
    Jp,
    Jnp,
    Jl,
    Jnl,
    Jle,
    Jnle,
    Test,
    Xchg,
    Mov,
    Lea,
    Pop,
    Push,
    Cbw,
    Cwd,
    CallFar,
    Wait,
    Pushf,
    Popf,
    Lahf,
    Sahf,
    Movsb,
    Movsw,
    Cmpsb,
    Cmpsw,
    Stosb,
    Stosw,
    Lodsb,
    Lodsw,
    Scasb,
    Scasw,
    Ret,
    Retf,
    Les,
    Lds,
    Int,
    Into,
    Iret,
    Rol,
    Ror,
    Rcl,
    Rcr,
    Shl,
    Shr,
    Sar,
    Aad,
    Aam,
    Xlat,
    Loop,
    Loope,
    Loopne,
    Jcxz,
    In,
    Out,
    Lock,
    Rep,
    Repne,
    Hlt,
    Cmc,
    CallNear,
    JmpNear,
    JmpFar,
    Not,
    Neg,
    Mul,
    Imul,
    Div,
    Idiv,
    Clc,
    Stc,
    Cli,
    Sti,
    Cld,
    Std,
    Inc,
}

pub enum BitOp {
    And,
    Xor,
    Or,
}

#[derive(Debug)]
pub struct Instruction {
    pub(crate) opcode: Opcode,
    pub(crate) dest: Operand,
    pub(crate) src: Operand,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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
    pub halt: bool
}

impl Cpu {
    pub fn init() -> Self {
        let mut cpu = Self {
            halt: false,
            prog_size: 0,
            regs: Registers::default(),
            mem: Mem::new(),
            seg_override: None,
        };
        cpu.regs.cs = 0xffff;
        cpu.regs.flags.set_from_u16(2);
        //println!("New Cpu: mem: {}", cpu.mem.size());
        cpu
    }

    pub fn test_mode(&mut self) {
        self.regs.cs = 0;
        self.regs.ds = 0;
        self.regs.set_ss(4096);
        self.regs.es = 0;
        self.regs.ip = 0;
        self.regs.sp = 4095;
    }

    pub fn fire(&mut self) {
        while let Some(i) = self.fetch() {
            self.execute(&i);
        }
    }

    pub fn get_seg_reg(&self, pos: u8) -> u16 {
        match pos & 0b11 {
            0 => self.regs.es,
            1 => self.regs.cs,
            2 => self.regs.ss,
            3 => self.regs.ds,
            4u8..=u8::MAX => panic!("invalid seg reg {}", pos),
        }
    }

    pub fn set_seg_reg(&mut self, pos: u8, val: u16) {
        match pos & 0b11 {
            0 => self.regs.es = val,
            1 => self.regs.cs = val,
            2 => self.regs.ss = val,
            3 => self.regs.ds = val,
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
            //self.seg_override = None;
            //unimplemented!("segment override");
            res
        } else {
            self.ea(&seg, offt)
        }
    }

    pub fn calc_op_displacement(&mut self, b1: Byte1, b2: Byte2) -> Operand {
        let mut offt = 0u32;
        match b2.modd() {
            0 => match b2.rm() {
                0 => {
                    offt = (self.regs.get_bx() + self.regs.get_si()) as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                    } else {
                        Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                    }
                }
                1 => {
                    offt = (self.regs.get_bx() + self.regs.get_di()) as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                    } else {
                        //offt = (self.regs.get_bx() + self.regs.get_di()) as u32;
                        Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                    }
                }
                2 => {
                    offt = (self.regs.get_bp() + self.regs.get_si()) as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ss, offt), offt)
                    } else {
                        //offt = (self.regs.get_bp() + self.regs.get_si()) as u32;
                        Operand::Mem8(self.get_segment_offset(Segment::Ss, offt), offt)
                    }
                }
                3 => {
                    offt = (self.regs.get_bp() + self.regs.get_di()) as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ss, offt), offt)
                    } else {
                        //offt = (self.regs.get_bp() + self.regs.get_di()) as u32;
                        Operand::Mem8(self.get_segment_offset(Segment::Ss, offt), offt)
                    }
                }
                4 => {
                    offt = (self.regs.get_si()) as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                    } else {
                        //offt = (self.regs.get_si()) as u32;
                        Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                    }
                }
                5 => {
                    offt = (self.regs.get_di()) as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                    } else {
                        //offt = (self.regs.get_di()) as u32;
                        Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                    }
                }
                6 => {
                    offt = self.mem.read_u16() as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                    } else {
                        Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                    }
                }
                7 => {
                    offt = (self.regs.get_bx()) as u32;
                    if b1.word() {
                        Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                    } else {
                        Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                    }
                }
                8..=u8::MAX => unreachable!(),
            },
            0b1 => {
                let disp = self.mem.read_u8() as u16;
                let res = match b2.rm() {
                    0 => {
                        offt = (self.regs.get_bx() + self.regs.get_si() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    1 => {
                        offt = (self.regs.get_bx() + self.regs.get_di() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    2 => {
                        offt = (self.regs.get_bp() + self.regs.get_si() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ss, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ss, offt), offt)
                        }
                    }
                    3 => {
                        offt = (self.regs.get_bp() + self.regs.get_di() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ss, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ss, offt), offt)
                        }
                    }
                    4 => {
                        offt = (self.regs.get_si() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    5 => {
                        offt = (self.regs.get_di() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    6 => {
                        offt = (self.regs.get_bp() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ss, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ss, offt), offt)
                        }
                    }
                    7 => {
                        offt = (self.regs.get_bx() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
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
                        offt = (self.regs.get_bx() + self.regs.get_si() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    1 => {
                        offt = (self.regs.get_bx() + self.regs.get_di() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    2 => {
                        offt = (self.regs.get_bp() + self.regs.get_si() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ss, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ss, offt), offt)
                        }
                    }
                    3 => {
                        offt = (self.regs.get_bp() + self.regs.get_di() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ss, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ss, offt), offt)
                        }
                    }
                    4 => {
                        offt = (self.regs.get_si() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    5 => {
                        offt = (self.regs.get_di() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    6 => {
                        let offt = self.mem.read_u16() as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
                        }
                    }
                    7 => {
                        offt = (self.regs.get_bx() + disp) as u32;
                        if b1.word() {
                            Operand::Mem16(self.get_segment_offset(Segment::Ds, offt), offt)
                        } else {
                            Operand::Mem8(self.get_segment_offset(Segment::Ds, offt), offt)
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
        self.mem.seek_to(self.code_addr(self.regs.ip) as u64);
        let old_pos = self.mem.pos();
        if self.regs.ip as u64 >= self.prog_size {
            return None;
        }

        let mut result = (Operand::Mem16(0, 0), Operand::Mem16(0, 0));
        let mut b1 = Byte1::new(self.mem.read_u8());

        //println!("========== Opcode: {}", b1.opcode());

        let mut b2 = Byte2::new(0);

        let res = match b1.opcode() {
            0 => {
                b2 = Byte2::new(self.mem.read_u8());
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
            2 => {
                b2 = Byte2::new(self.mem.read_u8());

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
                    opcode: Opcode::Or,
                    dest: result.0,
                    src: result.1,
                })
            }
            3 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Or,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Or,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::PushCs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            4 => {
                b2 = Byte2::new(self.mem.read_u8());
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
                    opcode: Opcode::Adc,
                    dest: result.0,
                    src: result.1,
                })
            }
            5 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Adc,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Adc,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::PushSs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::PopSs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            6 => {
                b2 = Byte2::new(self.mem.read_u8());
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
                    opcode: Opcode::Sbb,
                    dest: result.0,
                    src: result.1,
                })
            }
            7 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Sbb,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Sbb,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::PushDs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::PopDs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            8 => {
                b2 = Byte2::new(self.mem.read_u8());
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
                    opcode: Opcode::And,
                    dest: result.0,
                    src: result.1,
                })
            }
            9 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::And,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Add,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::OverrideEs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::Daa,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            10 => {
                b2 = Byte2::new(self.mem.read_u8());
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
                    opcode: Opcode::Sub,
                    dest: result.0,
                    src: result.1,
                })
            }
            11 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Sub,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Sub,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::OverrideCs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::Das,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            12 => {
                b2 = Byte2::new(self.mem.read_u8());
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
                    opcode: Opcode::Xor,
                    dest: result.0,
                    src: result.1,
                })
            }
            13 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Xor,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Xor,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::OverrideSs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::Aaa,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            14 => {
                b2 = Byte2::new(self.mem.read_u8());
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
                    opcode: Opcode::Cmp,
                    dest: result.0,
                    src: result.1,
                })
            }
            15 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Cmp,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Cmp,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::OverrideDs,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::Aas,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            16 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::IncAx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::IncCx,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::IncDx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::IncBx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },

            17 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::IncSp,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::IncBp,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::IncSi,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::IncDi,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            18 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::DecAx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::DecCx,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::DecDx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::DecBx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            19 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::DecSp,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::DecBp,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::DecSi,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::DecDi,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            20 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::PushAx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::PushCx,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::PushDx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::PushBx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            21 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::PushSp,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::PushBp,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::PushSi,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::PushDi,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            22 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::PopAx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::PopCx,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::PopDx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::PopBx,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            23 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::PopSp,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::PopBp,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::PopSi,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::PopDi,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            28 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Jo,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Jno,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::Jb,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::Jnb,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            29 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Jz,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Jnz,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::Jbe,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::Jnbe,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            30 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Js,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Jns,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::Jp,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::Jnp,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            31 => match b1.to_u8() & 0b11 {
                0 => Some(Instruction {
                    opcode: Opcode::Jl,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                1 => Some(Instruction {
                    opcode: Opcode::Jnl,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                2 => Some(Instruction {
                    opcode: Opcode::Jle,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                3 => Some(Instruction {
                    opcode: Opcode::Jnle,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                }),
                _ => unreachable!("instruction 3:2"),
            },
            32 => {
                b2 = Byte2::new(self.mem.read_u8());
                match b1.to_u8() & 0b11 {
                    0 => match b2.reg() {
                        0 => Some(Instruction {
                            opcode: Opcode::Add,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        1 => Some(Instruction {
                            opcode: Opcode::Or,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        2 => Some(Instruction {
                            opcode: Opcode::Adc,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        3 => Some(Instruction {
                            opcode: Opcode::Sbb,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        4 => Some(Instruction {
                            opcode: Opcode::And,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        5 => Some(Instruction {
                            opcode: Opcode::Sub,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        6 => Some(Instruction {
                            opcode: Opcode::Xor,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        7 => Some(Instruction {
                            opcode: Opcode::Cmp,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        _ => unimplemented!("op immediate"),
                    },
                    1 => match b2.reg() {
                        0 => Some(Instruction {
                            opcode: Opcode::Add,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        }),
                        1 => Some(Instruction {
                            opcode: Opcode::Or,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        }),
                        2 => Some(Instruction {
                            opcode: Opcode::Adc,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        }),
                        3 => Some(Instruction {
                            opcode: Opcode::Sbb,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        }),
                        4 => Some(Instruction {
                            opcode: Opcode::And,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        }),
                        5 => Some(Instruction {
                            opcode: Opcode::Sub,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        }),
                        6 => Some(Instruction {
                            opcode: Opcode::Xor,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        }),
                        7 => Some(Instruction {
                            opcode: Opcode::Cmp,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        }),
                        _ => unimplemented!("op immediate 16"),
                    },
                    2 => match b2.reg() {
                        0 => Some(Instruction {
                            opcode: Opcode::Add,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        2 => Some(Instruction {
                            opcode: Opcode::Adc,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        3 => Some(Instruction {
                            opcode: Opcode::Sbb,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        5 => Some(Instruction {
                            opcode: Opcode::Sub,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        7 => Some(Instruction {
                            opcode: Opcode::Cmp,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        }),
                        _ => unimplemented!("op immediate 16"),
                    },
                    3 => match b2.reg() {
                        0 => Some(Instruction {
                            opcode: Opcode::Add,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16((self.mem.read_i8() as i16) as u16),
                        }),
                        2 => Some(Instruction {
                            opcode: Opcode::Adc,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16((self.mem.read_i8() as i16) as u16),
                        }),
                        3 => Some(Instruction {
                            opcode: Opcode::Sbb,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16((self.mem.read_i8() as i16) as u16),
                        }),
                        5 => Some(Instruction {
                            opcode: Opcode::Sub,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16((self.mem.read_i8() as i16) as u16),
                        }),
                        7 => Some(Instruction {
                            opcode: Opcode::Cmp,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16((self.mem.read_i8() as i16) as u16),
                        }),
                        _ => unimplemented!("op immediate 16"),
                    },
                    _ => unimplemented!("op 32"),
                }
            }
            33 => {
                b2 = Byte2::new(self.mem.read_u8());
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
                    opcode: Opcode::Test,
                    dest: result.0,
                    src: result.1,
                })
            }
            34 => {
                b2 = Byte2::new(self.mem.read_u8());
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
                    opcode: Opcode::Mov,
                    dest: result.0,
                    src: result.1,
                })
            }
            35 => {
                b2 = Byte2::new(self.mem.read_u8());
                match b1.to_u8() & 0b11 {
                    0 => {
                        b1.set_word();
                        //println!("WORD: {}", b1.word());
                        match (b2.reg() & 0b100) > 0 {
                            false => Some(Instruction {
                                opcode: Opcode::Mov,
                                src: Operand::Seg(b2.reg() & 0b11),
                                dest: self.addr_mod(b1, b2),
                            }),
                            _ => unimplemented!("op immediate: 35"),
                        }
                    }
                    1 => {
                        b1.set_word();
                        Some(Instruction {
                            opcode: Opcode::Lea,
                            dest: Operand::Reg16(b2.reg()),
                            src: self.addr_mod(b1, b2),
                        })
                    }
                    2 => {
                        b1.set_word();
                        //println!("WORD: {}", b1.word());
                        match (b2.reg() & 0b100) > 0 {
                            false => Some(Instruction {
                                opcode: Opcode::Mov,
                                dest: Operand::Seg(b2.reg() & 0b11),
                                src: self.addr_mod(b1, b2),
                            }),
                            _ => unimplemented!("op immediate: 35"),
                        }
                    }
                    3 => match b2.reg() {
                        0 => Some(Instruction {
                            opcode: Opcode::Pop,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Reg8(0),
                        }),
                        _ => unreachable!("op: 35: reg: {}", b2.reg()),
                    },
                    _ => unimplemented!("op 35"),
                }
            }
            36 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Xchg,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(0),
                },
                1 => Instruction {
                    opcode: Opcode::Xchg,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(1),
                },
                2 => Instruction {
                    opcode: Opcode::Xchg,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
                3 => Instruction {
                    opcode: Opcode::Xchg,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(3),
                },
                _ => unreachable!(),
            }),
            37 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Xchg,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(4),
                },
                1 => Instruction {
                    opcode: Opcode::Xchg,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(5),
                },
                2 => Instruction {
                    opcode: Opcode::Xchg,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(6),
                },
                3 => Instruction {
                    opcode: Opcode::Xchg,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(7),
                },
                _ => unreachable!(),
            }),
            38 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Cbw,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(0),
                },
                1 => Instruction {
                    opcode: Opcode::Cwd,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(1),
                },
                2 => Instruction {
                    opcode: Opcode::CallFar,
                    dest: Operand::Imm16(self.mem.read_u16()),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                3 => Instruction {
                    opcode: Opcode::Wait,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(3),
                },
                _ => unreachable!(),
            }),
            39 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Pushf,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(0),
                },
                1 => Instruction {
                    opcode: Opcode::Popf,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(1),
                },
                2 => Instruction {
                    opcode: Opcode::Sahf,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
                3 => Instruction {
                    opcode: Opcode::Lahf,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(3),
                },
                _ => unreachable!(),
            }),
            40 => {
                let mut ea = self.mem.read_u16() as u32;
                ea = self.ea(&Segment::Ds, ea);
                Some(match b1.to_u8() & 0b11 {
                    0 => Instruction {
                        opcode: Opcode::Mov,
                        dest: Operand::Reg8(0),
                        src: Operand::Mem8(ea, 0),
                    },
                    1 => Instruction {
                        opcode: Opcode::Mov,
                        dest: Operand::Reg16(0),
                        src: Operand::Mem16(ea, 0),
                    },
                    2 => Instruction {
                        opcode: Opcode::Mov,
                        dest: Operand::Mem8(ea, 0),
                        src: Operand::Reg8(0),
                    },
                    3 => Instruction {
                        opcode: Opcode::Mov,
                        dest: Operand::Mem16(ea, 0),
                        src: Operand::Reg16(0),
                    },
                    _ => unreachable!(),
                })
            }
            41 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Movsb,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                1 => Instruction {
                    opcode: Opcode::Movsw,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                2 => Instruction {
                    opcode: Opcode::Cmpsb,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                3 => Instruction {
                    opcode: Opcode::Cmpsw,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                _ => unreachable!(),
            }),
            42 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Test,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                1 => Instruction {
                    opcode: Opcode::Test,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                2 => Instruction {
                    opcode: Opcode::Stosb,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                3 => Instruction {
                    opcode: Opcode::Stosw,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                _ => unreachable!(),
            }),
            43 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Lodsb,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                1 => Instruction {
                    opcode: Opcode::Lodsw,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                2 => Instruction {
                    opcode: Opcode::Scasb,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                3 => Instruction {
                    opcode: Opcode::Scasw,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                _ => unreachable!(),
            }),
            44 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                1 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg8(1),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                2 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg8(2),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                3 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg8(3),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                _ => unreachable!(),
            }),
            45 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg8(4),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                1 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg8(5),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                2 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg8(6),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                3 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg8(7),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                _ => unreachable!(),
            }),
            46 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                1 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg16(1),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                2 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg16(2),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                3 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg16(3),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                _ => unreachable!(),
            }),
            47 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg16(4),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                1 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg16(5),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                2 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg16(6),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                3 => Instruction {
                    opcode: Opcode::Mov,
                    dest: Operand::Reg16(7),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                _ => unreachable!(),
            }),
            48 => Some(match b1.to_u8() & 0b11 {
                2 => Instruction {
                    opcode: Opcode::Ret,
                    dest: Operand::Imm16(self.mem.read_u16()),
                    src: Operand::Reg8(0),
                },
                3 => Instruction {
                    opcode: Opcode::Ret,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                _ => unreachable!(),
            }),
            49 => {
                b2 = Byte2::new(self.mem.read_u8());
                Some(match b1.to_u8() & 0b11 {
                    0 => {
                        b1.set_word();
                        Instruction {
                            opcode: Opcode::Les,
                            dest: Operand::Reg16(b2.reg()),
                            src: self.calc_op_displacement(b1, b2),
                        }
                    }
                    1 => {
                        b1.set_word();
                        Instruction {
                            opcode: Opcode::Lds,
                            dest: Operand::Reg16(b2.reg()),
                            src: self.calc_op_displacement(b1, b2),
                        }
                    }
                    2 => match b2.reg() {
                        0 => Instruction {
                            opcode: Opcode::Mov,
                            dest: self.calc_op_displacement(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        },
                        _ => unreachable!("49:2"),
                    },
                    3 => match b2.reg() {
                        0 => Instruction {
                            opcode: Opcode::Mov,
                            dest: self.calc_op_displacement(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        },
                        _ => unreachable!("49:3"),
                    },
                    _ => unreachable!(),
                })
            }
            50 => Some(match b1.to_u8() & 0b11 {
                2 => Instruction {
                    opcode: Opcode::Retf,
                    dest: Operand::Imm16(self.mem.read_u16()),
                    src: Operand::Reg8(0),
                },
                3 => Instruction {
                    opcode: Opcode::Retf,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                _ => unreachable!(),
            }),
            51 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Int,
                    dest: Operand::Imm8(3),
                    src: Operand::Reg8(0),
                },
                1 => Instruction {
                    opcode: Opcode::Int,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Imm8(0),
                },
                2 => Instruction {
                    opcode: Opcode::Into,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                3 => Instruction {
                    opcode: Opcode::Iret,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                _ => unreachable!(),
            }),
            52 => {
                b2 = Byte2::new(self.mem.read_u8());
                match b1.to_u8() & 0b11 {
                    0 | 1 => match b2.reg() {
                        0 => Some(Instruction {
                            opcode: Opcode::Rol,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(1),
                        }),
                        1 => Some(Instruction {
                            opcode: Opcode::Ror,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(1),
                        }),
                        2 => Some(Instruction {
                            opcode: Opcode::Rcl,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(1),
                        }),
                        3 => Some(Instruction {
                            opcode: Opcode::Rcr,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(1),
                        }),
                        4 => Some(Instruction {
                            opcode: Opcode::Shl,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(1),
                        }),
                        5 => Some(Instruction {
                            opcode: Opcode::Shr,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(1),
                        }),
                        7 => Some(Instruction {
                            opcode: Opcode::Sar,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(1),
                        }),
                        _ => unimplemented!("op immediate"),
                    },
                    2 | 3 => match b2.reg() {
                        0 => Some(Instruction {
                            opcode: Opcode::Rol,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Reg8(1),
                        }),
                        1 => Some(Instruction {
                            opcode: Opcode::Ror,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Reg8(1),
                        }),
                        2 => Some(Instruction {
                            opcode: Opcode::Rcl,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Reg8(1),
                        }),
                        3 => Some(Instruction {
                            opcode: Opcode::Rcr,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Reg8(1),
                        }),
                        4 => Some(Instruction {
                            opcode: Opcode::Shl,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Reg8(1),
                        }),
                        5 => Some(Instruction {
                            opcode: Opcode::Shr,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Reg8(1),
                        }),
                        7 => Some(Instruction {
                            opcode: Opcode::Sar,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Reg8(1),
                        }),
                        _ => unimplemented!("op immediate 16"),
                    },
                    _ => unimplemented!("op 52"),
                }
            }
            53 => Some(match b1.to_u8() & 0b11 {
                0 => {
                    b2 = Byte2::new(self.mem.read_u8());
                    if b2.to_u8() == 0b1010 {
                        Instruction {
                            opcode: Opcode::Aam,
                            dest: Operand::Reg8(0),
                            src: Operand::Reg8(0),
                        }
                    } else {
                        panic!("AAM: wrong b2")
                    }
                }
                1 => {
                    b2 = Byte2::new(self.mem.read_u8());
                    if b2.to_u8() == 0b1010 {
                        Instruction {
                            opcode: Opcode::Aad,
                            dest: Operand::Reg8(0),
                            src: Operand::Reg8(0),
                        }
                    } else {
                        panic!("AAD: wrong b2")
                    }
                }
                3 => Instruction {
                    opcode: Opcode::Xlat,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg8(0),
                },
                _ => panic!("53"),
            }),
            56 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Loopne,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                },

                1 => Instruction {
                    opcode: Opcode::Loope,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                },
                2 => Instruction {
                    opcode: Opcode::Loop,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Reg8(0),
                },
                3 => Instruction {
                    opcode: Opcode::Jcxz,
                    dest: Operand::Imm8(self.mem.read_u8()),
                    src: Operand::Imm8(0),
                },
                _ => unreachable!(),
            }),
            57 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::In,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                1 => Instruction {
                    opcode: Opcode::In,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                2 => Instruction {
                    opcode: Opcode::Out,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                3 => Instruction {
                    opcode: Opcode::Out,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                _ => unreachable!(),
            }),
            58 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::CallNear,
                    dest: Operand::Reg8(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                },

                1 => Instruction {
                    opcode: Opcode::JmpNear,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                2 => Instruction {
                    opcode: Opcode::JmpFar,
                    dest: Operand::Imm16(self.mem.read_u16()),
                    src: Operand::Imm16(self.mem.read_u16()),
                },
                3 => Instruction {
                    opcode: Opcode::JmpNear,
                    dest: Operand::Reg16(0),
                    src: Operand::Imm8(self.mem.read_u8()),
                },
                _ => unreachable!(),
            }),
            59 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::In,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                1 => Instruction {
                    opcode: Opcode::In,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
                2 => Instruction {
                    opcode: Opcode::Out,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                3 => Instruction {
                    opcode: Opcode::Out,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
                _ => unreachable!(),
            }),
            60 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Lock,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                2 => Instruction {
                    opcode: Opcode::Repne,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                3 => Instruction {
                    opcode: Opcode::Rep,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
                _ => unreachable!(),
            }),
            61 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Hlt,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                1 => Instruction {
                    opcode: Opcode::Cmc,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                2 => {
                    b2 = Byte2::new(self.mem.read_u8());
                    match b2.reg() {
                        0 => Instruction {
                            opcode: Opcode::Test,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(self.mem.read_u8()),
                        },
                        2 => Instruction {
                            opcode: Opcode::Not,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        3 => Instruction {
                            opcode: Opcode::Neg,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        4 => Instruction {
                            opcode: Opcode::Mul,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        5 => Instruction {
                            opcode: Opcode::Imul,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        6 => Instruction {
                            opcode: Opcode::Div,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        7 => Instruction {
                            opcode: Opcode::Idiv,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        _ => unreachable!(),
                    }
                }
                3 => {
                    b2 = Byte2::new(self.mem.read_u8());
                    match b2.reg() {
                        0 => Instruction {
                            opcode: Opcode::Test,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm16(self.mem.read_u16()),
                        },
                        2 => Instruction {
                            opcode: Opcode::Not,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        3 => Instruction {
                            opcode: Opcode::Neg,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        4 => Instruction {
                            opcode: Opcode::Mul,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        5 => Instruction {
                            opcode: Opcode::Imul,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        6 => Instruction {
                            opcode: Opcode::Div,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        7 => Instruction {
                            opcode: Opcode::Idiv,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
                3 => Instruction {
                    opcode: Opcode::Rep,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
            }),
            62 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Clc,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                1 => Instruction {
                    opcode: Opcode::Stc,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
                2 => Instruction {
                    opcode: Opcode::Cli,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                3 => Instruction {
                    opcode: Opcode::Sti,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
                _ => unreachable!(),
            }),
            63 => Some(match b1.to_u8() & 0b11 {
                0 => Instruction {
                    opcode: Opcode::Cld,
                    dest: Operand::Reg8(0),
                    src: Operand::Reg16(2),
                },
                1 => Instruction {
                    opcode: Opcode::Std,
                    dest: Operand::Reg16(0),
                    src: Operand::Reg16(2),
                },
                2 => {
                    b2 = Byte2::new(self.mem.read_u8());
                    match b2.reg() {
                        0 => Instruction {
                            opcode: Opcode::Inc,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        1 => Instruction {
                            opcode: Opcode::Inc,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        _ => unreachable!(),
                    }
                }
                3 => {
                    b2 = Byte2::new(self.mem.read_u8());
                    match b2.reg() {
                        0 => Instruction {
                            opcode: Opcode::Inc,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        1 => Instruction {
                            opcode: Opcode::Inc,
                            dest: self.addr_mod(b1, b2),
                            src: Operand::Imm8(0),
                        },
                        2 => Instruction {
                            opcode: Opcode::CallNear,
                            dest: Operand::Imm8(0),
                            src: self.addr_mod(b1, b2),
                        },
                        3 => Instruction {
                            opcode: Opcode::CallFar,
                            src: Operand::Imm8(0),
                            dest: self.addr_mod(b1, b2),
                        },
                        4 => Instruction {
                            opcode: Opcode::JmpNear,
                            dest: Operand::Imm8(0),
                            src: self.addr_mod(b1, b2),
                        },
                        5 => Instruction {
                            opcode: Opcode::JmpFar,
                            dest: Operand::Imm8(0),
                            src: self.addr_mod(b1, b2),
                        },
                        6 => Instruction {
                            opcode: Opcode::Push,
                            dest: Operand::Imm8(0),
                            src: self.addr_mod(b1, b2),
                        },
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }),
            _ => unimplemented!("Opcode: {}", b1.opcode()),
        };
        self.regs.ip = self.regs.ip.wrapping_add((self.mem.pos() - old_pos) as u16);
        res
    }

    fn addr_mod(&mut self, b1: Byte1, b2: Byte2) -> Operand {
        match b2.modd() {
            3 => match b1.word() {
                true => Operand::Reg16(b2.rm()),
                false => Operand::Reg8(b2.rm()),
            },
            _ => self.calc_op_displacement(b1, b2),
        }
    }

    fn operand_value(&mut self, op: Operand) -> u16 {
        let pos = self.mem.pos();
        let val = match op {
            Operand::Mem16(i, _) => {
                self.mem.seek_to(i as u64);
                self.mem.read_u16()
            }
            Operand::Mem8(i, _) => {
                self.mem.seek_to(i as u64);
                self.mem.read_u8() as u16
            }
            Operand::Reg8(i) => self.get_reg(i, false),
            Operand::Reg16(i) => self.get_reg(i, true),
            Operand::Imm8(i) => i as u16,
            Operand::Imm16(i) => i,
            Operand::Seg(i) => self.get_seg_reg(i),
        };
        self.mem.seek_to(pos);
        val
    }

    pub fn write_mem_u16(&mut self, pos: u32, val: u16) {
        let p = self.mem.pos();
        self.mem.seek_to(pos as u64);
        self.mem.write_u16(val);
        self.mem.seek_to(p);
    }

    pub fn write_mem_u8(&mut self, pos: u32, val: u8) {
        let p = self.mem.pos();
        self.mem.seek_to(pos as u64);
        self.mem.write_u8(val);
        self.mem.seek_to(p);
    }

    pub fn read_mem_u16(&mut self, pos: u32) -> u16 {
        let p = self.mem.pos();
        self.mem.seek_to(pos as u64);
        let res = self.mem.read_u16();
        self.mem.seek_to(p);
        res
    }

    pub fn read_mem_u8(&mut self, pos: u32) -> u8 {
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

    fn sub(&mut self, d: Operand, s: Operand, sbb: bool, cmp: bool) {
        let dest = self.operand_value(d);
        let src = self.operand_value(s);

        let mut result = dest.wrapping_sub(src);

        if sbb {
            if (self.regs.flags.cf()) {
                result = result.wrapping_sub(1);
            }
        }

        self.regs.flags.clear_arith();

        if (Self::aux_sub(dest, src)) {
            self.regs.flags.set_af();
        }

        if Self::even_parity(result as u8) {
            self.regs.flags.set_pf();
        }

        if result == 0 {
            self.regs.flags.set_zf();
        }

        match d {
            Operand::Mem16(p, _) => {
                if (dest as i16).overflowing_sub(src as i16).1 {
                    self.regs.flags.set_of();
                }

                if (dest as u16).overflowing_sub(src as u16).1 {
                    self.regs.flags.set_cf();
                }

                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                if !cmp {
                    self.write_mem_u16(p, result)
                }
            }
            Operand::Mem8(p, _) => {
                if (dest as i8).overflowing_sub(src as i8).1 {
                    self.regs.flags.set_of();
                }

                if (dest as u8).overflowing_sub(src as u8).1 {
                    self.regs.flags.set_cf();
                }

                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }

                if !cmp {
                    self.write_mem_u8(p, result as u8)
                }
            }
            Operand::Reg8(r) => {
                if (dest as i8).overflowing_sub(src as i8).1 {
                    self.regs.flags.set_of();
                }

                if (dest as u8).overflowing_sub(src as u8).1 {
                    self.regs.flags.set_cf();
                }

                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }

                if !cmp {
                    self.set_reg(r, false, result)
                }
            }
            Operand::Reg16(r) => {
                if (dest as i16).overflowing_sub(src as i16).1 {
                    self.regs.flags.set_of();
                }

                if (dest as u16).overflowing_sub(src as u16).1 {
                    self.regs.flags.set_cf();
                }

                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                if !cmp {
                    self.set_reg(r, true, result)
                }
            }
            _ => unreachable!("Immediate destination"),
        }
    }

    fn dec(&mut self, d: Operand) {
        let dest = self.operand_value(d);
        let src = 1;

        let mut result = dest.wrapping_sub(src);
        self.regs.flags.clear_af();
        self.regs.flags.clear_sf();
        self.regs.flags.clear_zf();
        self.regs.flags.clear_of();
        self.regs.flags.clear_pf();

        if (Self::aux_sub(dest, src)) {
            self.regs.flags.set_af();
        }

        if Self::even_parity(result as u8) {
            self.regs.flags.set_pf();
        }

        if result == 0 {
            self.regs.flags.set_zf();
        }

        match d {
            Operand::Mem16(p, _) => {
                if (dest as i16).overflowing_sub(src as i16).1 {
                    self.regs.flags.set_of();
                }
                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.write_mem_u16(p, result)
            }
            Operand::Mem8(p, _) => {
                if (dest as i8).overflowing_sub(src as i8).1 {
                    self.regs.flags.set_of();
                }

                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.write_mem_u8(p, result as u8)
            }
            Operand::Reg8(r) => {
                if (dest as i8).overflowing_sub(src as i8).1 {
                    self.regs.flags.set_of();
                }

                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.set_reg(r, false, result)
            }
            Operand::Reg16(r) => {
                if (dest as i16).overflowing_sub(src as i16).1 {
                    self.regs.flags.set_of();
                }

                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.set_reg(r, true, result)
            }
            _ => unreachable!("Immediate destination"),
        }
    }

    fn add(&mut self, d: Operand, s: Operand, adc: bool) {
        let dest = self.operand_value(d);
        let src = self.operand_value(s);

        let mut result = dest.wrapping_add(src);

        if adc {
            if (self.regs.flags.cf()) {
                result = result.wrapping_add(1);
            }
        }
        self.regs.flags.clear_arith();

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
            Operand::Mem16(p, _) => {
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
            Operand::Mem8(p, _) => {
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

    fn inc(&mut self, d: Operand) {
        let dest = self.operand_value(d);
        let src = 1;

        let mut result = dest.wrapping_add(src);

        self.regs.flags.clear_af();
        self.regs.flags.clear_sf();
        self.regs.flags.clear_zf();
        self.regs.flags.clear_of();
        self.regs.flags.clear_pf();

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
            Operand::Mem16(p, _) => {
                if (dest as i16).overflowing_add(src as i16).1 {
                    self.regs.flags.set_of();
                }

                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.write_mem_u16(p, result)
            }
            Operand::Mem8(p, _) => {
                if (dest as i8).overflowing_add(src as i8).1 {
                    self.regs.flags.set_of();
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
                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }
                self.set_reg(r, false, result)
            }
            Operand::Reg16(r) => {
                if (dest as i16).overflowing_add(src as i16).1 {
                    self.regs.flags.set_of();
                }

                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }
                self.set_reg(r, true, result)
            }
            _ => unreachable!("Immediate destination"),
        }
    }

    pub fn bit_op(&mut self, d: Operand, s: Operand, op: BitOp, test: bool) {
        self.regs.flags.clear_arith();

        let dest = self.operand_value(d);
        let src = self.operand_value(s);

        let result = match op {
            BitOp::And => dest & src,
            BitOp::Xor => dest ^ src,
            BitOp::Or => dest | src,
        };

        if Self::even_parity(result as u8) {
            self.regs.flags.set_pf();
        }

        if result == 0 {
            self.regs.flags.set_zf();
        }

        match d {
            Operand::Mem16(p, _) => {
                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                if !test {
                    self.write_mem_u16(p, result)
                }
            }
            Operand::Mem8(p, _) => {
                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }
                if !test {
                    self.write_mem_u8(p, result as u8)
                }
            }
            Operand::Reg8(r) => {
                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }
                if !test {
                    self.set_reg(r, false, result)
                }
            }
            Operand::Reg16(r) => {
                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                if !test {
                    self.set_reg(r, true, result)
                }
            }
            _ => unreachable!("Immediate destination"),
        }
    }

    fn daa(&mut self) {
        let mut al = self.regs.get_al();
        if al & 0b1111 > 9 || self.regs.flags.af() {
            al = al.wrapping_add(6);
            self.regs.flags.set_af();
        }

        if al > 0x9f || self.regs.flags.cf() {
            al = al.wrapping_add(0x60);
            self.regs.flags.set_cf();
        }

        if (al == 0) {
            self.regs.flags.set_zf();
        }

        if al & 128 > 0 {
            self.regs.flags.set_sf();
        }

        if al & 128 != self.regs.get_al() & 128 {
            self.regs.flags.set_of();
        }

        if Self::even_parity(al) {
            self.regs.flags.set_pf();
        }

        self.regs.set_al(al);
    }

    fn aaa(&mut self) {
        let mut al = self.regs.get_al();
        let mut ah = self.regs.get_ah();
        if al & 0b1111 > 9 || self.regs.flags.af() {
            al = al.wrapping_add(6);
            ah = ah.wrapping_add(1);
            self.regs.flags.set_af();
            self.regs.flags.set_cf();
        } else {
            self.regs.flags.clear_af();
            self.regs.flags.clear_cf();
        }
        self.regs.set_al(al & 0b1111);
        self.regs.set_ah(ah);
    }

    fn das(&mut self) {
        let mut al = self.regs.get_al();
        if al & 0b1111 > 9 || self.regs.flags.af() {
            al = al.wrapping_sub(6);
            self.regs.flags.set_af();
        }

        if al > 0x9f || self.regs.flags.cf() {
            al = al.wrapping_sub(0x60);
            self.regs.flags.set_cf();
        }

        if (al == 0) {
            self.regs.flags.set_zf();
        }

        if al & 128 > 0 {
            self.regs.flags.set_sf();
        }

        if al & 128 != self.regs.get_al() & 128 {
            self.regs.flags.set_of();
        }

        if Self::even_parity(al) {
            self.regs.flags.set_pf();
        }

        self.regs.set_al(al);
    }

    fn aas(&mut self) {
        let mut al = self.regs.get_al();
        let mut ah = self.regs.get_ah();
        if al & 0b1111 > 9 || self.regs.flags.af() {
            al = al.wrapping_sub(6);
            ah = ah.wrapping_sub(1);
            self.regs.flags.set_af();
            self.regs.flags.set_cf();
        } else {
            self.regs.flags.clear_af();
            self.regs.flags.clear_cf();
        }
        self.regs.set_al(al & 0b1111);
        self.regs.set_ah(ah);
    }

    fn push(&mut self, val: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        self.write_mem_u16(self.stack_addr(self.regs.sp), val);
    }

    fn pop(&mut self) -> u16 {
        let v = self.read_mem_u16(self.stack_addr(self.regs.sp));
        self.regs.sp = self.regs.sp.wrapping_add(2);
        v
    }

    fn pushf(&mut self) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        self.write_mem_u16(self.stack_addr(self.regs.sp), self.regs.flags.to_u16());
    }

    fn popf(&mut self) {
        let v = self.read_mem_u16(self.stack_addr(self.regs.sp));
        self.regs.flags.set_from_u16(v);
        self.regs.sp = self.regs.sp.wrapping_add(2);
    }

    fn pop2(&mut self, inst: &Instruction) {
        let val = self.read_mem_u16(self.stack_addr(self.regs.sp));
        self.regs.sp = self.regs.sp.wrapping_add(2);

        match inst.dest {
            Operand::Mem16(p, _) => {
                self.write_mem_u16(p, val);
            }
            Operand::Reg16(r) => {
                self.set_reg(r, true, val);
            }
            _ => panic!("invalid pop dest"),
        }
    }

    fn adjust_ip_short(&mut self, val: u8) {
        let v = val as i8;
        if v >= 0 {
            self.regs.ip = self.regs.ip.wrapping_add(v.abs() as u16);
        } else {
            self.regs.ip = self.regs.ip.wrapping_sub(v.abs() as u16);
        }
    }

    fn adjust_ip_long(&mut self, val: u16) {
        let v = val as i16;
        if v >= 0 {
            self.regs.ip = self.regs.ip.wrapping_add(v.abs() as u16);
        } else {
            self.regs.ip = self.regs.ip.wrapping_sub(v.abs() as u16);
        }
    }

    fn exchg(&mut self, inst: &Instruction) {
        let mut d = 0u16;
        let mut s = 0u16;
        match inst.dest {
            Operand::Mem16(i, _) => {
                if let Operand::Reg16(r) = inst.src {
                    d = self.read_mem_u16(i);
                    s = self.get_reg(r, true);
                    self.set_reg(r, true, d);
                    self.write_mem_u16(i, s);
                } else {
                    panic!("src must be reg 16")
                };
            }
            Operand::Mem8(i, _) => {
                if let Operand::Reg8(r) = inst.src {
                    d = self.read_mem_u8(i) as u16;
                    s = self.get_reg(r, false);
                    self.set_reg(r, false, d);
                    self.write_mem_u8(i, s as u8);
                } else {
                    panic!("src must be reg 8")
                };
            }
            Operand::Reg8(r) => match inst.src {
                Operand::Mem8(i, _) => {
                    d = self.read_mem_u8(i) as u16;
                    s = self.get_reg(r, false);
                    self.set_reg(r, false, d);
                    self.write_mem_u8(i, s as u8);
                }
                Operand::Reg8(reg) => {
                    d = self.get_reg(r, false);
                    s = self.get_reg(reg, false);
                    self.set_reg(reg, false, d);
                    self.set_reg(r, false, s);
                }
                _ => panic!("exchg with immediate or non 8bit"),
            },
            Operand::Reg16(r) => match inst.src {
                Operand::Mem16(i, _) => {
                    d = self.read_mem_u16(i);
                    s = self.get_reg(r, true);
                    self.set_reg(r, true, d as u16);
                    self.write_mem_u16(i, s);
                }
                Operand::Reg16(reg) => {
                    d = self.get_reg(r, true);
                    s = self.get_reg(reg, true);
                    self.set_reg(reg, true, d);
                    self.set_reg(r, true, s);
                }
                _ => panic!("exchg with immediate or non 16bit"),
            },
            _ => panic!("exchg with immediate"),
        }
    }

    fn mov(&mut self, inst: &Instruction) {
        let mut d = 0u16;
        let mut s = 0u16;
        match inst.dest {
            Operand::Mem16(i, _) => {
                match inst.src {
                    Operand::Reg16(r) => {
                        //d = self.read_mem_u16(i);
                        s = self.get_reg(r, true);
                        //self.set_reg(r, true, d);
                        self.write_mem_u16(i, s);
                    }
                    Operand::Seg(r) => {
                        s = self.get_seg_reg(r);
                        self.write_mem_u16(i, s);
                    }
                    Operand::Imm16(imm) => {
                        self.write_mem_u16(i, imm);
                    }
                    _ => panic!("src must be reg 16"),
                }
            }
            Operand::Mem8(i, _) => {
                if let Operand::Reg8(r) = inst.src {
                    //d = self.read_mem_u8(i) as u16;
                    s = self.get_reg(r, false);
                    //self.set_reg(r, false, d);
                    self.write_mem_u8(i, s as u8);
                } else if let Operand::Imm8(imm) = inst.src {
                    self.write_mem_u8(i, imm as u8);
                } else {
                    panic!("src must be reg 8")
                };
            }
            Operand::Reg8(r) => match inst.src {
                Operand::Mem8(i, _) => {
                    d = self.read_mem_u8(i) as u16;
                    //s = self.get_reg(r, false);
                    self.set_reg(r, false, d);
                    //self.write_mem_u8(i, s as u8);
                }
                Operand::Reg8(reg) => {
                    //d = self.get_reg(r, false);
                    s = self.get_reg(reg, false);
                    //self.set_reg(reg, false, d);
                    self.set_reg(r, false, s);
                }
                Operand::Imm8(im) => {
                    self.set_reg(r, false, im as u16);
                }
                _ => panic!("exchg with immediate or non 8bit"),
            },
            Operand::Reg16(r) => match inst.src {
                Operand::Mem16(i, _) => {
                    d = self.read_mem_u16(i);
                    //s = self.get_reg(r, true);
                    self.set_reg(r, true, d as u16);
                    //self.write_mem_u16(i, s);
                }
                Operand::Reg16(reg) => {
                    //d = self.get_reg(r, true);
                    s = self.get_reg(reg, true);
                    //self.set_reg(reg, true, d);
                    self.set_reg(r, true, s);
                }
                Operand::Seg(reg) => {
                    //d = self.get_reg(r, true);
                    s = self.get_seg_reg(reg);
                    //self.set_reg(reg, true, d);
                    self.set_reg(r, true, s);
                }
                Operand::Imm16(im) => {
                    self.set_reg(r, true, im);
                }
                _ => panic!("mov to immediate or non 16bit"),
            },
            Operand::Seg(r) => {
                let val = match inst.src {
                    Operand::Reg16(r) => self.get_reg(r, true),
                    Operand::Mem16(m, _) => self.read_mem_u16(m),
                    _ => panic!("mov seg invalid\n"),
                };
                self.set_seg_reg(r, val);
            }
            _ => panic!("mov to immediate"),
        }
    }

    fn lea(&mut self, inst: &Instruction) {
        match inst.dest {
            Operand::Reg16(r) => match inst.src {
                Operand::Mem16(_, m) => {
                    self.set_reg(r, true, m as u16);
                }
                _ => unreachable!("Lea: invalid op"),
            },
            _ => unreachable!("Lea: invalid op"),
        }
    }

    fn cbw(&mut self) {
        if (self.regs.get_al() & 0b10000000) > 0 {
            self.regs.set_ah(255);
        } else {
            self.regs.set_ah(0);
        }
    }

    fn cwd(&mut self) {
        if (self.regs.get_ah() & 0b10000000) > 0 {
            self.regs.set_dx(0xffff);
        } else {
            self.regs.set_dx(0);
        }
    }

    fn lahf(&mut self) {
        let v = self.regs.flags.to_u16();
        self.regs.set_ah(v as u8);
    }

    fn sahf(&mut self) {
        let mut v = self.regs.flags.to_u16() & 0xff00;
        v |= self.regs.get_ah() as u16;
        self.regs.flags.set_from_u16(v);
    }

    fn movsb(&mut self) {
        let mut dest = self.extra_addr(self.regs.di);
        let mut src = self.data_addr(self.regs.si);
        let val = self.read_mem_u8(src);
        self.write_mem_u8(dest, val);
        if !self.regs.flags.df() {
            self.regs.di = self.regs.di.wrapping_add(1);
            self.regs.si = self.regs.di.wrapping_add(1);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(1);
            self.regs.si = self.regs.di.wrapping_sub(1);
        }
    }

    fn movsw(&mut self) {
        let mut dest = self.extra_addr(self.regs.di);
        let mut src = self.data_addr(self.regs.si);
        let val = self.read_mem_u16(src);
        self.write_mem_u16(dest, val);
        if !self.regs.flags.df() {
            self.regs.di = self.regs.di.wrapping_add(2);
            self.regs.si = self.regs.di.wrapping_add(2);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(2);
            self.regs.si = self.regs.di.wrapping_sub(2);
        }
    }

    fn cmpsb(&mut self) {
        let mut destt = self.extra_addr(self.regs.di);
        let mut srcc = self.data_addr(self.regs.si);

        let a = self.read_mem_u8(srcc);
        let b = self.read_mem_u8(destt);

        let result = a.wrapping_sub(b);

        self.regs.flags.clear_arith();

        if (Self::aux_sub(a as u16, b as u16)) {
            self.regs.flags.set_af();
        }

        if Self::even_parity(result as u8) {
            self.regs.flags.set_pf();
        }

        if result == 0 {
            self.regs.flags.set_zf();
        }

        if (a as i8).overflowing_sub(b as i8).1 {
            self.regs.flags.set_of();
        }

        if (a as u8).overflowing_sub(b as u8).1 {
            self.regs.flags.set_cf();
        }

        if result & !0b01111111 > 0 {
            self.regs.flags.set_sf();
        }

        if !self.regs.flags.df() {
            self.regs.di = self.regs.di.wrapping_add(1);
            self.regs.si = self.regs.di.wrapping_add(1);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(1);
            self.regs.si = self.regs.di.wrapping_sub(1);
        }
    }

    fn scasb(&mut self) {
        let mut destt = self.extra_addr(self.regs.di);

        let a = self.read_mem_u8(destt);
        let b = self.regs.get_ah();

        let result = a.wrapping_sub(b);

        self.regs.flags.clear_arith();

        if (Self::aux_sub(a as u16, b as u16)) {
            self.regs.flags.set_af();
        }

        if Self::even_parity(result as u8) {
            self.regs.flags.set_pf();
        }

        if result == 0 {
            self.regs.flags.set_zf();
        }

        if (a as i8).overflowing_sub(b as i8).1 {
            self.regs.flags.set_of();
        }

        if (a as u8).overflowing_sub(b as u8).1 {
            self.regs.flags.set_cf();
        }

        if result & !0b01111111 > 0 {
            self.regs.flags.set_sf();
        }

        if !self.regs.flags.df() {
            self.regs.di = self.regs.di.wrapping_add(1);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(1);
        }
    }

    fn scasw(&mut self) {
        let mut destt = self.extra_addr(self.regs.di);

        let a = self.read_mem_u16(destt);
        let b = self.regs.get_ax();

        let result = a.wrapping_sub(b);

        self.regs.flags.clear_arith();

        if (Self::aux_sub(a as u16, b as u16)) {
            self.regs.flags.set_af();
        }

        if Self::even_parity(result as u8) {
            self.regs.flags.set_pf();
        }

        if result == 0 {
            self.regs.flags.set_zf();
        }

        if (a as i16).overflowing_sub(b as i16).1 {
            self.regs.flags.set_of();
        }

        if (a as u16).overflowing_sub(b as u16).1 {
            self.regs.flags.set_cf();
        }

        if result & !0b01111111 > 0 {
            self.regs.flags.set_sf();
        }

        if !self.regs.flags.df() {
            self.regs.di = self.regs.di.wrapping_add(2);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(2);
        }
    }

    fn cmpsw(&mut self) {
        let mut destt = self.extra_addr(self.regs.di);
        let mut srcc = self.data_addr(self.regs.si);

        let a = self.read_mem_u16(srcc);
        let b = self.read_mem_u16(destt);

        let result = a.wrapping_sub(b);

        self.regs.flags.clear_arith();

        if (Self::aux_sub(a as u16, b as u16)) {
            self.regs.flags.set_af();
        }

        if Self::even_parity(result as u8) {
            self.regs.flags.set_pf();
        }

        if result == 0 {
            self.regs.flags.set_zf();
        }

        if (a as i16).overflowing_sub(b as i16).1 {
            self.regs.flags.set_of();
        }

        if (a as u16).overflowing_sub(b as u16).1 {
            self.regs.flags.set_cf();
        }

        if result & !0b01111111_11111111 > 0 {
            self.regs.flags.set_sf();
        }

        if !self.regs.flags.df() {
            self.regs.di = self.regs.di.wrapping_add(1);
            self.regs.si = self.regs.di.wrapping_add(1);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(1);
            self.regs.si = self.regs.di.wrapping_sub(1);
        }
    }

    fn stosb(&mut self) {
        let mut destt = self.extra_addr(self.regs.di);
        self.write_mem_u8(destt, self.regs.get_al());

        if !self.regs.flags.df() {
            self.regs.di = self.regs.di.wrapping_add(1);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(1);
        }
    }

    fn stosw(&mut self) {
        let mut destt = self.extra_addr(self.regs.di);
        self.write_mem_u16(destt, self.regs.get_ax());

        if !self.regs.flags.df() {
            self.regs.di = self.regs.di.wrapping_add(2);
        } else {
            self.regs.di = self.regs.di.wrapping_sub(2);
        }
    }

    fn lodsb(&mut self) {
        let mut src = self.data_addr(self.regs.si);
        let val = self.read_mem_u8(src);
        self.regs.set_al(val);
        if !self.regs.flags.df() {
            self.regs.si = self.regs.si.wrapping_add(1);
        } else {
            self.regs.si = self.regs.si.wrapping_sub(1);
        }
    }

    fn lodsw(&mut self) {
        let mut src = self.data_addr(self.regs.si);
        let val = self.read_mem_u16(src);
        self.regs.set_ax(val);
        if !self.regs.flags.df() {
            self.regs.si = self.regs.si.wrapping_add(2);
        } else {
            self.regs.si = self.regs.si.wrapping_sub(2);
        }
    }

    fn ret(&mut self, inst: &Instruction) {
        self.regs.ip = self.pop();
        if let Operand::Imm16(im) = inst.dest {
            self.regs.sp = self.regs.sp.wrapping_add(im);
        }
    }

    fn retf(&mut self, inst: &Instruction) {
        self.regs.ip = self.pop();
        self.regs.cs = self.pop();
        if let Operand::Imm16(im) = inst.dest {
            self.regs.sp = self.regs.sp.wrapping_add(im);
        }
    }

    fn les(&mut self, inst: &Instruction) {
        match inst.dest {
            Operand::Reg16(r) => match inst.src {
                Operand::Mem16(m, _) => {
                    let mut w = self.read_mem_u16(m);
                    self.set_reg(r, true, w);
                    w = self.read_mem_u16(m.wrapping_add(2));
                    self.regs.es = w;
                }
                _ => panic!("les: invalid op"),
            },
            _ => panic!("les: invalid op"),
        }
    }

    fn lds(&mut self, inst: &Instruction) {
        match inst.dest {
            Operand::Reg16(r) => match inst.src {
                Operand::Mem16(m, _) => {
                    let mut w = self.read_mem_u16(m);
                    self.set_reg(r, true, w);
                    w = self.read_mem_u16(m.wrapping_add(2));
                    self.regs.ds = w;
                }
                _ => panic!("les: invalid op"),
            },
            _ => panic!("les: invalid op"),
        }
    }

    fn rot8(&mut self, dest: u8, times: u8, left: bool) -> u8 {
        let mut rn = 0u8;
        let res = if left {
            rn = (dest).rotate_left(times as u32);
            if times > 0 && (rn & 1) > 0 {
                self.regs.flags.set_cf();
            }
            rn
        } else {
            rn = (dest).rotate_right(times as u32);
            if times > 0 && (rn & 128) > 0 {
                self.regs.flags.set_cf();
            }
            rn
        };

        if res & !0b01111111 != dest & !0b01111111 {
            self.regs.flags.set_of();
        }
        res
    }

    fn rot16(&mut self, dest: u16, times: u8, left: bool) -> u16 {
        let mut rn = 0u16;
        let res = if left {
            rn = (dest).rotate_left(times as u32);
            if times > 0 && (rn & 1) > 0 {
                self.regs.flags.set_cf();
            }
            rn
        } else {
            rn = (dest).rotate_right(times as u32);
            if times > 0 && (rn & !0b01111111_11111111) > 0 {
                self.regs.flags.set_cf();
            }
            rn
        };

        if res & !0b01111111_11111111 != dest & !0b01111111_11111111 {
            self.regs.flags.set_of();
        }
        res
    }

    fn rotate(&mut self, inst: &Instruction, left: bool) {
        let times = match inst.src {
            Operand::Imm8(imm) => imm,
            Operand::Reg8(1) => self.regs.get_cl(),
            _ => unreachable!("Rol: invalid ops"),
        };

        let dest = self.operand_value(inst.dest);
        self.regs.flags.clear_cf();
        self.regs.flags.clear_of();
        match inst.dest {
            Operand::Reg16(id) => {
                let val = self.rot16(dest, times, left);
                self.set_reg(id, true, val);
            }
            Operand::Mem16(pos, _) => {
                let val = self.rot16(dest, times, left);
                self.write_mem_u16(pos, val);
            }
            Operand::Reg8(id) => {
                let val = self.rot8(dest as u8, times, left);
                self.set_reg(id, false, val as u16);
            }
            Operand::Mem8(pos, _) => {
                let val = self.rot8(dest as u8, times, left);
                self.write_mem_u8(pos, val);
            }
            _ => unreachable!(),
        }
    }

    fn rotcf8(&mut self, dest: u8, times: u8, left: bool) -> u8 {
        let oldcf = self.regs.flags.cf();
        self.regs.flags.clear_cf();
        self.regs.flags.clear_of();
        let mut rn = 0u8;
        let res = if left {
            rn = (dest).rotate_left(times as u32);
            if times > 0 && (rn & 1) > 0 {
                self.regs.flags.set_cf();
            }

            rn &= !1;
            rn |= oldcf as u8;

            rn
        } else {
            rn = (dest).rotate_right(times as u32);
            if times > 0 && (rn & 128) > 0 {
                self.regs.flags.set_cf();
            }

            rn &= !128;
            rn |= (oldcf as u8) << 7;
            rn
        };

        if res & !0b01111111 != dest & !0b01111111 {
            self.regs.flags.set_of();
        }
        res
    }

    fn rotcf16(&mut self, dest: u16, times: u8, left: bool) -> u16 {
        let oldcf = self.regs.flags.cf();

        self.regs.flags.clear_cf();
        self.regs.flags.clear_of();

        let mut rn = 0u16;
        let res = if left {
            rn = (dest).rotate_left(times as u32);
            if times > 0 && (rn & 1) > 0 {
                self.regs.flags.set_cf();
            }
            rn &= !1;
            rn |= oldcf as u16;

            rn
        } else {
            rn = (dest).rotate_right(times as u32);
            if times > 0 && (rn & !0b01111111_11111111) > 0 {
                self.regs.flags.set_cf();
            }
            rn &= 0x7fff;
            rn |= (oldcf as u16) << 15;
            rn
        };

        if res & !0b01111111_11111111 != dest & !0b01111111_11111111 {
            self.regs.flags.set_of();
        }
        res
    }

    fn rotate_cf(&mut self, inst: &Instruction, left: bool) {
        let times = match inst.src {
            Operand::Imm8(imm) => imm,
            Operand::Reg8(1) => self.regs.get_cl(),
            _ => unreachable!("Rol: invalid ops"),
        };

        let dest = self.operand_value(inst.dest);
        match inst.dest {
            Operand::Reg16(id) => {
                let val = self.rotcf16(dest, times, left);
                self.set_reg(id, true, val);
            }
            Operand::Mem16(pos, _) => {
                let val = self.rotcf16(dest, times, left);
                self.write_mem_u16(pos, val);
            }
            Operand::Reg8(id) => {
                let val = self.rotcf8(dest as u8, times, left);
                self.set_reg(id, false, val as u16);
            }
            Operand::Mem8(pos, _) => {
                let val = self.rotcf8(dest as u8, times, left);
                self.write_mem_u8(pos, val);
            }
            _ => unreachable!(),
        }
    }

    fn sh8(&mut self, val: u8, left: bool) -> u8 {
        self.regs.flags.clear_cf();
        self.regs.flags.clear_of();

        let res = if left {
            if (val & 128) > 0 {
                self.regs.flags.set_cf();
            }
            val.wrapping_shl(1)
        } else {
            if (val & 1) > 0 {
                self.regs.flags.set_cf();
            }
            val.wrapping_shr(1)
        };
        if (val & 128) != (res & 128) {
            self.regs.flags.set_of();
        }
        res
    }

    fn sh16(&mut self, val: u16, left: bool) -> u16 {
        self.regs.flags.clear_cf();
        self.regs.flags.clear_of();

        let res = if left {
            if (val & 0x8000) > 0 {
                self.regs.flags.set_cf();
            }
            val.wrapping_shl(1)
        } else {
            if (val & 1) > 0 {
                self.regs.flags.set_cf();
            }
            val.wrapping_shr(1)
        };
        if (val & 0x8000) != (res & 0x8000) {
            self.regs.flags.set_of();
        }
        res
    }

    fn shift(&mut self, inst: &Instruction, left: bool) {
        let times = self.operand_value(inst.src);
        let dest = self.operand_value(inst.dest);
        for i in 0..times {
            match inst.dest {
                Operand::Reg16(id) => {
                    let val = self.sh16(dest, left);
                    self.set_reg(id, true, val);
                }
                Operand::Mem16(pos, _) => {
                    let val = self.sh16(dest, left);
                    self.write_mem_u16(pos, val);
                }
                Operand::Reg8(id) => {
                    let val = self.sh8(dest as u8, left);
                    self.set_reg(id, false, val as u16);
                }
                Operand::Mem8(pos, _) => {
                    let val = self.sh8(dest as u8, left);
                    self.write_mem_u8(pos, val);
                }
                _ => unreachable!(),
            }
        }
    }

    fn shal8(&mut self, val: u8) -> u8 {
        self.regs.flags.clear_cf();
        self.regs.flags.clear_of();

        let mut res = {
            if (val & 1) > 0 {
                self.regs.flags.set_cf();
            }
            val.wrapping_shr(1)
        };

        res |= (val & 128);

        if (val & 128) != (res & 128) {
            self.regs.flags.set_of();
        }
        res
    }

    fn shal16(&mut self, val: u16) -> u16 {
        self.regs.flags.clear_cf();
        self.regs.flags.clear_of();

        let mut res = {
            if (val & 1) > 0 {
                self.regs.flags.set_cf();
            }
            val.wrapping_shr(1)
        };

        res |= (val & 0x8000);

        if (val & 0x8000) != (res & 0x8000) {
            self.regs.flags.set_of();
        }
        res
    }

    fn shalr(&mut self, inst: &Instruction) {
        let times = self.operand_value(inst.src);
        let dest = self.operand_value(inst.dest);
        for i in 0..times {
            match inst.dest {
                Operand::Reg16(id) => {
                    let val = self.shal16(dest);
                    self.set_reg(id, true, val);
                }
                Operand::Mem16(pos, _) => {
                    let val = self.shal16(dest);
                    self.write_mem_u16(pos, val);
                }
                Operand::Reg8(id) => {
                    let val = self.shal8(dest as u8);
                    self.set_reg(id, false, val as u16);
                }
                Operand::Mem8(pos, _) => {
                    let val = self.shal8(dest as u8);
                    self.write_mem_u8(pos, val);
                }
                _ => unreachable!(),
            }
        }
    }

    fn aad(&mut self) {
        let al = self
            .regs
            .get_ah()
            .wrapping_mul(10)
            .wrapping_add(self.regs.get_al());
        if al == 0 {
            self.regs.flags.set_zf();
        }

        if (al & 128) > 0 {
            self.regs.flags.set_sf();
        }

        if Self::even_parity(al) {
            self.regs.flags.set_pf();
        }

        self.regs.set_al(al);
        self.regs.set_ah(0);
    }

    fn aam(&mut self) {
        let ah = self.regs.get_al().wrapping_div(10);
        let al = self.regs.get_al().wrapping_rem(10);

        self.regs.set_al(al);
        self.regs.set_ah(ah);

        let ax = self.regs.ax;

        if (ax & 0x8000) > 0 {
            self.regs.flags.set_sf();
        }

        if ax == 0 {
            self.regs.flags.set_zf();
        }

        if Self::even_parity(al) {
            self.regs.flags.set_pf();
        }
    }

    fn xlat(&mut self) {
        let offt = self.regs.bx + self.regs.get_al() as u16;
        let byte = self.read_mem_u8(self.data_addr(offt));
        self.regs.set_al(byte);
    }

    fn loopp(&mut self, inst: &Instruction) {
        self.regs.cx = self.regs.cx.wrapping_sub(1);
        if self.regs.cx != 0 {
            if let Operand::Imm8(i) = inst.dest {
                self.adjust_ip_short(i);
            } else {
                unreachable!()
            }
        }
    }

    fn loope(&mut self, inst: &Instruction) {
        self.regs.cx = self.regs.cx.wrapping_sub(1);
        if self.regs.cx != 0 && self.regs.flags.zf() {
            if let Operand::Imm8(i) = inst.dest {
                self.adjust_ip_short(i);
            } else {
                unreachable!()
            }
        }
    }

    fn loopne(&mut self, inst: &Instruction) {
        self.regs.cx = self.regs.cx.wrapping_sub(1);
        if self.regs.cx != 0 && !self.regs.flags.zf() {
            if let Operand::Imm8(i) = inst.dest {
                self.adjust_ip_short(i);
            } else {
                unreachable!()
            }
        }
    }

    fn jcxz(&mut self, inst: &Instruction) {
        if self.regs.cx == 0 {
            if let Operand::Imm8(i) = inst.dest {
                self.adjust_ip_short(i);
            } else {
                unreachable!()
            }
        }
    }

    fn jmp_near(&mut self, inst: &Instruction) {
        match inst.src {
            Operand::Imm16(imm) => {
                self.adjust_ip_long(imm);
            }
            Operand::Imm8(imm) => {
                self.adjust_ip_short(imm);
            }
            Operand::Reg16(r) => {
                self.regs.ip = self.get_reg(r, true);
            }
            _ => unreachable!(),
        }
    }

    fn call_near(&mut self, inst: &Instruction) {
        match inst.src {
            Operand::Imm16(imm) => {
                self.push(self.regs.ip);
                self.adjust_ip_long(imm);
            }
            Operand::Mem16(pos, _) => {
                self.push(self.regs.ip);
                self.regs.ip = self.read_mem_u16(pos);
            }
            Operand::Reg16(r) => {
                self.push(self.regs.ip);
                self.regs.ip = self.get_reg(r, true);
            }
            _ => unreachable!(),
        }
    }

    fn call_far(&mut self, inst: &Instruction) {
        self.push(self.regs.cs);
        self.push(self.regs.ip);
        //self.regs.ip = self.pop();
        //self.regs.cs = self.pop();

        match inst.dest {
            Operand::Imm16(ip) => match inst.src {
                Operand::Imm16(seg) => {
                    self.regs.ip = ip;
                    self.regs.cs = seg
                }
                _ => unreachable!(),
            },
            Operand::Mem16(pos, _) => {
                self.regs.ip = self.read_mem_u16(pos);
                self.regs.cs = self.read_mem_u16(pos.wrapping_add(2))
            }
            _ => unreachable!(),
        }
    }

    fn jmp_far(&mut self, inst: &Instruction) {
        match inst.src {
            Operand::Imm16(imm) => match inst.dest {
                Operand::Imm16(imm2) => {
                    self.regs.ip = imm2;
                    self.regs.cs = imm;
                }
                _ => unreachable!(),
            },
            Operand::Mem16(pos, _) => {
                self.regs.ip = self.read_mem_u16(pos);
                self.regs.cs = self.read_mem_u16(pos.wrapping_add(2))
            }
            _ => unreachable!(),
        }
    }

    fn rep(&mut self) {
        if let Some(instr) = self.fetch() {
            match instr.opcode {
                Opcode::Lodsb
                | Opcode::Lodsw
                | Opcode::Stosb
                | Opcode::Stosw
                | Opcode::Movsb
                | Opcode::Movsw => {
                    while self.regs.cx != 0 {
                        println!("cx: [{}], [{}] {:?}", self.regs.cx, self.regs.ip, instr);
                        self.execute(&instr);
                        self.regs.cx = self.regs.cx.wrapping_sub(1);
                    }
                }
                Opcode::Cmpsw | Opcode::Cmpsb | Opcode::Scasw | Opcode::Scasb => self.repe(&instr),
                _ => {
                    self.execute(&instr);
                }
            }
        }
    }

    fn repe(&mut self, instr: &Instruction) {
        while self.regs.cx != 0 {
            println!("cx: [{}], [{}] {:?}", self.regs.cx, self.regs.ip, instr);
            self.execute(instr);
            self.regs.cx = self.regs.cx.wrapping_sub(1);
            if !self.regs.flags.zf() {
                break;
            }
        }
    }

    fn repne(&mut self) {
        if let Some(instr) = self.fetch() {
            match instr.opcode {
                Opcode::Cmpsw | Opcode::Cmpsb | Opcode::Scasw | Opcode::Scasb => {
                    while self.regs.cx != 0 {
                        println!("cx: [{}], [{}] {:?}", self.regs.cx, self.regs.ip, instr);
                        self.execute(&instr);
                        self.regs.cx = self.regs.cx.wrapping_sub(1);
                        if self.regs.flags.zf() {
                            break;
                        }
                    }
                }
                _ => {
                    self.execute(&instr);
                }
            }
        }
    }

    fn push_mem(&mut self, inst: &Instruction) {
        match inst.src {
            Operand::Mem16(pos, _) => {
                let val = self.read_mem_u16(pos);
                self.push(val);
            }
            _ => unreachable!(),
        }
    }

    fn mul(&mut self, inst: &Instruction) {
        self.regs.flags.clear_of();
        self.regs.flags.clear_cf();
        match inst.dest {
            Operand::Reg8(_) | Operand::Mem8(_, _) => {
                let op = match inst.dest {
                    Operand::Reg8(r) => self.get_reg(r, false),
                    Operand::Mem8(pos, _) => self.read_mem_u8(pos) as u16,
                    _ => unreachable!(),
                };
                self.regs.ax = (self.regs.get_al() as u16).wrapping_mul(op);
                if self.regs.get_ah() != 0 {
                    self.regs.flags.set_of();
                    self.regs.flags.set_cf();
                }
            }
            Operand::Mem16(_, _) | Operand::Reg16(_) => {
                let op = match inst.dest {
                    Operand::Reg16(r) => self.get_reg(r, true),
                    Operand::Mem16(pos, _) => self.read_mem_u16(pos),
                    _ => unreachable!(),
                };
                let res = (self.regs.ax as u32).wrapping_mul(op as u32);
                self.regs.ax = res as u16;
                self.regs.dx = (res >> 16) as u16;
                if self.regs.dx != 0 {
                    self.regs.flags.set_of();
                    self.regs.flags.set_cf();
                }
            }
            _ => unreachable!(),
        }
    }

    fn imul(&mut self, inst: &Instruction) {
        self.regs.flags.clear_of();
        self.regs.flags.clear_cf();
        match inst.dest {
            Operand::Reg8(_) | Operand::Mem8(_, _) => {
                let op = match inst.dest {
                    Operand::Reg8(r) => self.get_reg(r, false) as i16,
                    Operand::Mem8(pos, _) => self.read_mem_u8(pos) as i16,
                    _ => unreachable!(),
                };
                self.regs.ax = (self.regs.get_al() as i16).wrapping_mul(op) as u16;
                if self.regs.get_ah() != 0xff {
                    self.regs.flags.set_of();
                    self.regs.flags.set_cf();
                }
            }
            Operand::Mem16(_, _) | Operand::Reg16(_) => {
                let op = match inst.dest {
                    Operand::Reg16(r) => self.get_reg(r, true) as i16,
                    Operand::Mem16(pos, _) => self.read_mem_u16(pos) as i16,
                    _ => unreachable!(),
                };
                let res = (self.regs.ax as i32).wrapping_mul(op as i32);
                self.regs.ax = res as u16;
                self.regs.dx = (res >> 16) as u16;

                if self.regs.dx != 0xffff {
                    self.regs.flags.set_of();
                    self.regs.flags.set_cf();
                }
            }
            _ => unreachable!(),
        }
    }

    fn idiv(&mut self, inst: &Instruction) {
        match inst.dest {
            Operand::Reg8(_) | Operand::Mem8(_, _) => {
                let op = match inst.dest {
                    Operand::Reg8(r) => self.get_reg(r, false) as i8,
                    Operand::Mem8(pos, _) => self.read_mem_u8(pos) as i8,
                    _ => unreachable!(),
                };
                let res = (self.regs.get_al() as i8).wrapping_div(op);
                let resmod = (self.regs.get_al() as i8).wrapping_rem(op);
                self.regs.set_ah(resmod as u8);
                self.regs.set_al(res as u8);
            }
            Operand::Mem16(_, _) | Operand::Reg16(_) => {
                let op = match inst.dest {
                    Operand::Reg16(r) => self.get_reg(r, true) as i16,
                    Operand::Mem16(pos, _) => self.read_mem_u16(pos) as i16,
                    _ => unreachable!(),
                };
                let res = (self.regs.ax as i16).wrapping_div(op);
                let resmod = (self.regs.ax as i16).wrapping_rem(op);

                self.regs.ax = res as u16;
                self.regs.dx = resmod as u16;
            }
            _ => unreachable!(),
        }
    }

    fn div(&mut self, inst: &Instruction) {
        match inst.dest {
            Operand::Reg8(_) | Operand::Mem8(_, _) => {
                let op = match inst.dest {
                    Operand::Reg8(r) => self.get_reg(r, false) as u8,
                    Operand::Mem8(pos, _) => self.read_mem_u8(pos),
                    _ => unreachable!(),
                };
                let res = (self.regs.get_al()).wrapping_div(op);
                let resmod = (self.regs.get_al()).wrapping_rem(op);
                self.regs.set_ah(resmod);
                self.regs.set_al(res);
            }
            Operand::Mem16(_, _) | Operand::Reg16(_) => {
                let op = match inst.dest {
                    Operand::Reg16(r) => self.get_reg(r, true),
                    Operand::Mem16(pos, _) => self.read_mem_u16(pos),
                    _ => unreachable!(),
                };
                let res = (self.regs.ax).wrapping_div(op);
                let resmod = (self.regs.ax).wrapping_rem(op);

                self.regs.ax = res;
                self.regs.dx = resmod;
            }
            _ => unreachable!(),
        }
    }

    fn not(&mut self, inst: &Instruction) {
        match inst.dest {
            Operand::Reg8(r) => {
                let d = self.get_reg(r, false);
                self.set_reg(r, false, !d);
            }
            Operand::Mem8(pos, _) => {
                let d = self.read_mem_u8(pos);
                self.write_mem_u8(pos, !d);
            }
            Operand::Reg16(r) => {
                let d = self.get_reg(r, true);
                self.set_reg(r, true, !d);
            }
            Operand::Mem16(pos, _) => {
                let d = self.read_mem_u16(pos);
                self.write_mem_u16(pos, !d);
            }
            _ => unreachable!(),
        };
    }

    fn neg(&mut self, inst: &Instruction) {
        match inst.dest {
            Operand::Reg8(r) => {
                let d = self.get_reg(r, false);
                self.set_reg(r, false, d.wrapping_neg());
            }
            Operand::Mem8(pos, _) => {
                let d = self.read_mem_u8(pos);
                self.write_mem_u8(pos, d.wrapping_neg());
            }
            Operand::Reg16(r) => {
                let d = self.get_reg(r, true);
                self.set_reg(r, true, d.wrapping_neg());
            }
            Operand::Mem16(pos, _) => {
                let d = self.read_mem_u16(pos);
                self.write_mem_u16(pos, d.wrapping_neg());
            }
            _ => unreachable!(),
        };
    }

    fn int(&mut self, inst: &Instruction) {
        self.push(self.regs.flags.to_u16());
        self.push(self.regs.cs);
        self.push(self.regs.ip);

        self.regs.flags.clear_if();

        match inst.dest {
            Operand::Imm8(imm) => {
                let offt = (imm as u32).wrapping_mul(4);
                self.regs.ip = self.read_mem_u16(offt);
                self.regs.cs = self.read_mem_u16(offt.wrapping_add(2));
            }
            _ => unreachable!(),
        }
    }

    fn into(&mut self, inst: &Instruction) {
        self.push(self.regs.flags.to_u16());
        self.push(self.regs.cs);
        self.push(self.regs.ip);
        if self.regs.flags.of() {
            self.regs.flags.clear_if();
            let offt = (4u32).wrapping_mul(4);
            self.regs.ip = self.read_mem_u16(offt);
            self.regs.cs = self.read_mem_u16(offt.wrapping_add(2));
        }
    }

    fn iret(&mut self, inst: &Instruction) {
        self.regs.ip = self.pop();
        self.regs.cs = self.pop();
        let f = self.pop();
        self.regs.flags.set_from_u16(f);
    }

    fn hlt(&mut self) {
        self.halt = true;
    }

    pub fn execute(&mut self, inst: &Instruction) {
        match inst.opcode {
            Opcode::Or => self.bit_op(inst.dest, inst.src, BitOp::Or, false),
            Opcode::Add => self.add(inst.dest, inst.src, false),
            Opcode::Adc => self.add(inst.dest, inst.src, true),
            Opcode::Sbb => self.sub(inst.dest, inst.src, true, false),
            Opcode::Sub => self.sub(inst.dest, inst.src, false, false),
            Opcode::Cmp => self.sub(inst.dest, inst.src, false, true),
            Opcode::PushEs => {
                self.push(self.regs.es);
            }
            Opcode::PushCs => {
                self.push(self.regs.cs);
            }
            Opcode::PopEs => {
                self.regs.es = self.pop();
            }
            Opcode::PushSs => {
                self.push(self.regs.ss);
            }
            Opcode::PopSs => {
                self.regs.ss = self.pop();
            }
            Opcode::PushDs => {
                self.push(self.regs.ds);
            }
            Opcode::PopDs => {
                self.regs.ds = self.pop();
            }
            Opcode::And => self.bit_op(inst.dest, inst.src, BitOp::And, false),
            Opcode::Xor => self.bit_op(inst.dest, inst.src, BitOp::Xor, false),
            Opcode::OverrideCs | Opcode::OverrideDs | Opcode::OverrideEs | Opcode::OverrideSs => {
                match inst.opcode {
                    Opcode::OverrideEs => self.seg_override = Some(Segment::Es),
                    Opcode::OverrideCs => self.seg_override = Some(Segment::Cs),
                    Opcode::OverrideSs => self.seg_override = Some(Segment::Ss),
                    Opcode::OverrideDs => self.seg_override = Some(Segment::Ds),
                    _ => unreachable!(),
                }
                return;
            }
            Opcode::Daa => self.daa(),
            Opcode::Aaa => self.aaa(),
            Opcode::Das => self.das(),
            Opcode::Aas => self.aas(),
            Opcode::IncAx => self.inc(Operand::Reg16(0)),
            Opcode::IncCx => self.inc(Operand::Reg16(1)),
            Opcode::IncBx => self.inc(Operand::Reg16(2)),
            Opcode::IncDx => self.inc(Operand::Reg16(3)),
            Opcode::IncSp => self.inc(Operand::Reg16(4)),
            Opcode::IncBp => self.inc(Operand::Reg16(5)),
            Opcode::IncSi => self.inc(Operand::Reg16(6)),
            Opcode::IncDi => self.inc(Operand::Reg16(7)),
            Opcode::DecAx => self.dec(Operand::Reg16(0)),
            Opcode::DecCx => self.dec(Operand::Reg16(1)),
            Opcode::DecBx => self.dec(Operand::Reg16(2)),
            Opcode::DecDx => self.dec(Operand::Reg16(3)),
            Opcode::DecSp => self.dec(Operand::Reg16(4)),
            Opcode::DecBp => self.dec(Operand::Reg16(5)),
            Opcode::DecSi => self.dec(Operand::Reg16(6)),
            Opcode::DecDi => self.dec(Operand::Reg16(7)),
            Opcode::PushAx => self.push(self.regs.ax),
            Opcode::PushCx => self.push(self.regs.cx),
            Opcode::PushBx => self.push(self.regs.bx),
            Opcode::PushDx => self.push(self.regs.dx),
            Opcode::PushSp => self.push(self.regs.sp),
            Opcode::PushBp => self.push(self.regs.bp),
            Opcode::PushSi => self.push(self.regs.si),
            Opcode::PushDi => self.push(self.regs.di),
            Opcode::PopAx => self.regs.ax = self.pop(),
            Opcode::PopCx => self.regs.cx = self.pop(),
            Opcode::PopBx => self.regs.bx = self.pop(),
            Opcode::PopDx => self.regs.dx = self.pop(),
            Opcode::PopSp => self.regs.sp = self.pop(),
            Opcode::PopBp => self.regs.bp = self.pop(),
            Opcode::PopSi => self.regs.si = self.pop(),
            Opcode::PopDi => self.regs.di = self.pop(),
            Opcode::Jo => {
                if let Operand::Imm8(i) = inst.dest {
                    if self.regs.flags.of() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jno => {
                if let Operand::Imm8(i) = inst.dest {
                    if !self.regs.flags.of() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jb => {
                if let Operand::Imm8(i) = inst.dest {
                    if self.regs.flags.cf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jnb => {
                if let Operand::Imm8(i) = inst.dest {
                    if !self.regs.flags.cf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jz => {
                if let Operand::Imm8(i) = inst.dest {
                    if self.regs.flags.zf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jnz => {
                if let Operand::Imm8(i) = inst.dest {
                    if !self.regs.flags.zf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jbe => {
                if let Operand::Imm8(i) = inst.dest {
                    if self.regs.flags.cf() || self.regs.flags.zf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jnbe => {
                if let Operand::Imm8(i) = inst.dest {
                    if !self.regs.flags.cf() && !self.regs.flags.zf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Js => {
                if let Operand::Imm8(i) = inst.dest {
                    if self.regs.flags.sf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jns => {
                if let Operand::Imm8(i) = inst.dest {
                    if !self.regs.flags.sf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jp => {
                if let Operand::Imm8(i) = inst.dest {
                    if self.regs.flags.pf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jnp => {
                if let Operand::Imm8(i) = inst.dest {
                    if !self.regs.flags.pf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jl => {
                if let Operand::Imm8(i) = inst.dest {
                    if self.regs.flags.sf() != self.regs.flags.of() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jnl => {
                if let Operand::Imm8(i) = inst.dest {
                    if self.regs.flags.sf() == self.regs.flags.of() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jle => {
                if let Operand::Imm8(i) = inst.dest {
                    if (self.regs.flags.sf() != self.regs.flags.of()) || self.regs.flags.zf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Jnle => {
                if let Operand::Imm8(i) = inst.dest {
                    if (self.regs.flags.sf() == self.regs.flags.of()) || !self.regs.flags.zf() {
                        self.adjust_ip_short(i);
                    }
                }
            }
            Opcode::Test => self.bit_op(inst.dest, inst.src, BitOp::And, true),
            Opcode::Xchg => self.exchg(&inst),
            Opcode::Mov => self.mov(&inst),
            Opcode::Lea => self.lea(&inst),
            Opcode::Pop => self.pop2(&inst),
            Opcode::Push => self.push_mem(&inst),
            Opcode::Cbw => self.cbw(),
            Opcode::Cwd => self.cwd(),
            Opcode::CallFar => self.call_far(&inst),
            Opcode::Wait => todo!(),
            Opcode::Pushf => self.pushf(),
            Opcode::Popf => self.popf(),
            Opcode::Lahf => self.lahf(),
            Opcode::Sahf => self.sahf(),
            Opcode::Movsb => self.movsb(),
            Opcode::Movsw => self.movsw(),
            Opcode::Cmpsw => self.cmpsw(),
            Opcode::Cmpsb => self.cmpsb(),
            Opcode::Stosb => self.stosb(),
            Opcode::Lodsb => self.lodsb(),
            Opcode::Scasb => self.scasb(),
            Opcode::Stosw => self.stosw(),
            Opcode::Lodsw => self.lodsw(),
            Opcode::Scasw => self.scasw(),
            Opcode::Ret => self.ret(&inst),
            Opcode::Retf => self.retf(&inst),
            Opcode::Les => self.les(&inst),
            Opcode::Lds => self.lds(&inst),
            Opcode::Int => self.int(&inst),
            Opcode::Into => self.into(&inst),
            Opcode::Iret => self.iret(&inst),
            Opcode::Rol => self.rotate(&inst, true),
            Opcode::Ror => self.rotate(&inst, false),
            Opcode::Rcl => self.rotate_cf(&inst, true),
            Opcode::Rcr => self.rotate_cf(&inst, false),
            Opcode::Shl => self.shift(&inst, true),
            Opcode::Shr => self.shift(&inst, false),
            Opcode::Sar => self.shalr(&inst),
            Opcode::Aad => self.aad(),
            Opcode::Aam => self.aam(),
            Opcode::Xlat => self.xlat(),
            Opcode::Loop => self.loopp(&inst),
            Opcode::Loope => self.loope(&inst),
            Opcode::Loopne => self.loopne(&inst),
            Opcode::Jcxz => self.jcxz(&inst),
            Opcode::In => todo!(),
            Opcode::Out => todo!(),
            Opcode::Lock => todo!(),
            Opcode::Rep => self.rep(),
            Opcode::Repne => self.repne(),
            Opcode::Hlt => self.hlt(),
            Opcode::Cmc => todo!(),
            Opcode::CallNear => self.call_near(&inst),
            Opcode::JmpNear => self.jmp_near(&inst),
            Opcode::JmpFar => self.jmp_far(&inst),
            Opcode::Not => self.not(&inst),
            Opcode::Neg => self.neg(&inst),
            Opcode::Mul => self.mul(&inst),
            Opcode::Imul => self.imul(&inst),
            Opcode::Div => self.div(&inst),
            Opcode::Idiv => self.idiv(&inst),
            Opcode::Clc => self.regs.flags.clear_cf(),
            Opcode::Stc => self.regs.flags.set_cf(),
            Opcode::Cli => self.regs.flags.clear_if(),
            Opcode::Sti => self.regs.flags.set_if(),
            Opcode::Cld => self.regs.flags.clear_df(),
            Opcode::Std => self.regs.flags.set_df(),
            Opcode::Inc => self.inc(inst.dest),
        }
        self.seg_override = None;
    }

    // program will be cut
    pub fn load_code(&mut self, path: &str) {
        if let Ok(mut file) = File::open(path) {

            self.mem.seek_to(self.code_addr(0) as u64);
            while self.mem.pos() < 1024 {
                let mut buf = [0u8];
                if let Ok(0) = file.read(&mut buf) {
                    break;
                }
                self.mem.write_u8(buf[0]);
            }
            self.prog_size = self.mem.pos();
        } else {
            println!("Failed to open file: {}", path);
            exit(1);
        }
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

    pub fn load_code_stdin(&mut self) {
        self.mem.seek_to(self.code_addr(0) as u64);
        let mut it = std::io::stdin().bytes();
        while self.mem.pos() < 1024 {
            if let Some(rs) = it.next() {
                match rs {
                    Ok(v) => self.mem.write_u8(v),
                    Err(_)=> panic!("error reading stdin"),
                }
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
}
