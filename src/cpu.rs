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
}

pub enum BitOp {
    And,
    Xor,
    Or,
}

#[derive(Debug)]
pub struct Instruction {
    opcode: Opcode,
    dest: Operand,
    src: Operand,
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
            //self.seg_override = None;
            //unimplemented!("segment override");
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
        self.mem.seek_to(self.code_addr(self.regs.ip) as u64);
        let old_pos = self.mem.pos();
        if self.regs.ip as u64 >= self.prog_size {
            return None;
        }

        let mut result = (Operand::Mem16(0), Operand::Mem16(0));
        let b1 = Byte1::new(self.mem.read_u8());

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
            _ => unimplemented!("Opcode: {}", b1.opcode()),
        };
        self.regs.ip = self.regs.ip.wrapping_add((self.mem.pos() - old_pos) as u16);
        res
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

    fn read_mem_u8(&mut self, pos: u32) -> u8 {
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
            Operand::Mem16(p) => {
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
            Operand::Mem8(p) => {
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

    pub fn bit_op(&mut self, d: Operand, s: Operand, op: BitOp) {
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
            Operand::Mem16(p) => {
                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.write_mem_u16(p, result)
            }
            Operand::Mem8(p) => {
                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }

                self.write_mem_u8(p, result as u8)
            }
            Operand::Reg8(r) => {
                if result & !0b01111111 > 0 {
                    self.regs.flags.set_sf();
                }
                self.set_reg(r, false, result)
            }
            Operand::Reg16(r) => {
                if result & !0b01111111_11111111 > 0 {
                    self.regs.flags.set_sf();
                }
                self.set_reg(r, true, result)
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

    fn adjust_ip_short(&mut self, val: u8) {
        let v = val as i8;
        if v >= 0 {
            self.regs.ip = self.regs.ip.wrapping_add(v.abs() as u16);
        } else {
            self.regs.ip = self.regs.ip.wrapping_sub(v.abs() as u16);
        }
    }

    pub fn execute(&mut self, inst: &Instruction) {
        match inst.opcode {
            Opcode::Or => self.bit_op(inst.dest, inst.src, BitOp::Or),
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
            Opcode::And => self.bit_op(inst.dest, inst.src, BitOp::And),
            Opcode::Xor => self.bit_op(inst.dest, inst.src, BitOp::Xor),
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

            Opcode::IncAx => self.add(Operand::Reg16(0), Operand::Imm16(1), false),
            Opcode::IncCx => self.add(Operand::Reg16(1), Operand::Imm16(1), false),
            Opcode::IncBx => self.add(Operand::Reg16(2), Operand::Imm16(1), false),
            Opcode::IncDx => self.add(Operand::Reg16(3), Operand::Imm16(1), false),
            Opcode::IncSp => self.add(Operand::Reg16(4), Operand::Imm16(1), false),
            Opcode::IncBp => self.add(Operand::Reg16(5), Operand::Imm16(1), false),
            Opcode::IncSi => self.add(Operand::Reg16(6), Operand::Imm16(1), false),
            Opcode::IncDi => self.add(Operand::Reg16(7), Operand::Imm16(1), false),

            Opcode::DecAx => self.sub(Operand::Reg16(0), Operand::Imm16(1), false, false),
            Opcode::DecCx => self.sub(Operand::Reg16(1), Operand::Imm16(1), false, false),
            Opcode::DecBx => self.sub(Operand::Reg16(2), Operand::Imm16(1), false, false),
            Opcode::DecDx => self.sub(Operand::Reg16(3), Operand::Imm16(1), false, false),
            Opcode::DecSp => self.sub(Operand::Reg16(4), Operand::Imm16(1), false, false),
            Opcode::DecBp => self.sub(Operand::Reg16(5), Operand::Imm16(1), false, false),
            Opcode::DecSi => self.sub(Operand::Reg16(6), Operand::Imm16(1), false, false),
            Opcode::DecDi => self.sub(Operand::Reg16(7), Operand::Imm16(1), false, false),
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
            //_ => unimplemented!(),
        }

        self.seg_override = None;
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
    fn cmp() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 0;
        cpu.execute(&Instruction {
            opcode: Opcode::Cmp,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert_eq!(cpu.regs.flags.zf(), true);
        assert_eq!(cpu.regs.flags.zf(), true);

        cpu.regs.ax = 1;
        cpu.regs.cx = 2;
        cpu.execute(&Instruction {
            opcode: Opcode::Cmp,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(1),
        });

        assert_eq!(cpu.regs.flags.cf(), true);
        assert!(cpu.regs.flags.sf());
    }

    #[test]
    fn aas() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 0x2ff;
        cpu.execute(&Instruction {
            opcode: Opcode::Aas,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert_eq!(cpu.regs.get_ah(), 1);
        assert_eq!(cpu.regs.get_al(), 9);
    }

    #[test]
    fn aaa() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 0xf;
        cpu.execute(&Instruction {
            opcode: Opcode::Aaa,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert_eq!(cpu.regs.get_ah(), 1);
        assert_eq!(cpu.regs.get_al(), 5);
    }

    #[test]
    fn das() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 0xff;
        cpu.execute(&Instruction {
            opcode: Opcode::Das,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert_eq!(cpu.regs.get_al(), 0x99);
        assert!(cpu.regs.flags.cf())
    }

    #[test]
    fn daa() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 0xf;
        cpu.execute(&Instruction {
            opcode: Opcode::Daa,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert_eq!(cpu.regs.get_al(), 0x15)
    }

    #[test]
    fn ov_ss() {
        let mut cpu = Cpu::init();
        cpu.execute(&Instruction {
            opcode: Opcode::OverrideSs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        cpu.regs.set_cs(0);
        cpu.regs.set_ds(1024);
        cpu.regs.set_ss(4096);
        cpu.regs.set_es(2048);

        assert_eq!(cpu.seg_override, Some(cpu::Segment::Ss));

        assert_eq!(
            cpu.get_segment_offset(cpu::Segment::Cs, 0),
            cpu.get_segment_offset(cpu::Segment::Ds, 0)
        );
        assert_eq!(
            cpu.get_segment_offset(cpu::Segment::Es, 0),
            cpu.get_segment_offset(cpu::Segment::Ss, 0)
        );
    }

    #[test]
    fn and() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 255;
        cpu.execute(&Instruction {
            opcode: Opcode::And,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(2),
        });
        assert_eq!(cpu.regs.ax, 0);
        assert!(cpu.regs.flags.zf());
        assert!(cpu.regs.flags.pf());
    }

    #[test]
    fn or2() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 255;
        cpu.execute(&Instruction {
            opcode: Opcode::Or,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(1),
        });
        assert_eq!(cpu.regs.ax, 255);

        assert!(!cpu.regs.flags.zf());
        assert!(cpu.regs.flags.pf());
    }

    #[test]
    fn xor() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 255;
        cpu.execute(&Instruction {
            opcode: Opcode::Xor,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.regs.ax, 0);
        assert!(cpu.regs.flags.zf());
        assert!(cpu.regs.flags.pf());
    }

    #[test]
    fn push_pop_ds() {
        let mut cpu = Cpu::init();
        cpu.mem.seek_to(cpu.code_addr(0) as u64);
        cpu.regs.set_cs(0);
        cpu.regs.set_ds(0);
        cpu.regs.set_ss(4096);
        cpu.regs.set_es(32);
        cpu.regs.sp = 64;
        cpu.regs.ds = 128;
        cpu.execute(&Instruction {
            opcode: Opcode::PushDs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.regs.sp, 62);
        assert_eq!(cpu.read_mem_u16(cpu.stack_addr(cpu.regs.sp)), 128);
        cpu.write_mem_u16(cpu.stack_addr(cpu.regs.sp), 64);
        let sp = cpu.regs.sp;
        cpu.execute(&Instruction {
            opcode: Opcode::PopDs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.regs.ds, 64);
        assert_eq!(cpu.regs.sp - sp, 2);
    }

    #[test]
    fn sbb() {
        let mut cpu = Cpu::init();
        cpu.mem.seek_to(cpu.code_addr(0) as u64);
        cpu.regs.set_ss(4096);

        cpu.regs.flags.set_cf();
        assert!(cpu.regs.flags.cf());

        cpu.execute(&Instruction {
            opcode: Opcode::Sbb,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert_eq!(cpu.regs.ax as i8, -1);
    }

    #[test]
    fn push_pop_ss() {
        let mut cpu = Cpu::init();
        cpu.mem.seek_to(cpu.code_addr(0) as u64);
        cpu.regs.set_cs(0);
        cpu.regs.set_ds(0);
        cpu.regs.set_ss(4096);
        cpu.regs.set_es(32);
        cpu.regs.sp = 64;
        cpu.regs.ss = 128;
        cpu.execute(&Instruction {
            opcode: Opcode::PushSs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.regs.sp, 62);
        assert_eq!(cpu.read_mem_u16(cpu.stack_addr(cpu.regs.sp)), 128);
        cpu.write_mem_u16(cpu.stack_addr(cpu.regs.sp), 64);
        let sp = cpu.regs.sp;
        cpu.execute(&Instruction {
            opcode: Opcode::PopSs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.regs.ss, 64);
        assert_eq!(cpu.regs.sp - sp, 2);
    }

    #[test]
    fn wa() {
        let mut a: i32 = 90;
        a = a.wrapping_add(1);
        assert!(a == 91);
    }

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
        cpu.regs.sp = 64;
        cpu.execute(&Instruction {
            opcode: Opcode::PushEs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.regs.sp, 62);
        assert_eq!(cpu.read_mem_u16(cpu.stack_addr(cpu.regs.sp)), 2);
        cpu.write_mem_u16(cpu.stack_addr(cpu.regs.sp), 64);
        let sp = cpu.regs.sp;
        cpu.execute(&Instruction {
            opcode: Opcode::PopEs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });
        assert_eq!(cpu.regs.es, 64);
        assert_eq!(cpu.regs.sp - sp, 2);
    }

    #[test]
    fn or() {
        let mut cpu = Cpu::init();
        cpu.regs.ax = 0b11;
        cpu.regs.cx = 0b1100;
        cpu.execute(&Instruction {
            opcode: Opcode::Or,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(1),
        });

        assert_eq!(cpu.regs.ax, 0b1111);
        assert!(cpu.regs.flags.pf());
        assert!(!cpu.regs.flags.zf());

        cpu.regs.ax = 0b00;
        cpu.regs.cx = 0b00;
        cpu.execute(&Instruction {
            opcode: Opcode::Or,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(1),
        });

        assert_eq!(cpu.regs.ax, 0b0);
        assert!(cpu.regs.flags.pf());
        assert!(cpu.regs.flags.zf());
    }

    #[test]
    fn push_cs() {
        let mut cpu = Cpu::init();
        cpu.mem.seek_to(cpu.code_addr(0) as u64);
        cpu.regs.set_ss(4096);
        cpu.regs.cs = 90;
        cpu.execute(&Instruction {
            opcode: Opcode::PushCs,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert_eq!(cpu.read_mem_u16(cpu.stack_addr(cpu.regs.sp)), 90);
    }

    #[test]
    fn adc() {
        let mut cpu = Cpu::init();
        cpu.mem.seek_to(cpu.code_addr(0) as u64);
        cpu.regs.set_ss(4096);
        //cpu.regs.cs = 90;
        cpu.regs.ax = 255;
        cpu.execute(&Instruction {
            opcode: Opcode::Add,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        cpu.regs.ax = 0;

        cpu.execute(&Instruction {
            opcode: Opcode::Adc,
            dest: Operand::Reg8(0),
            src: Operand::Reg8(0),
        });

        assert_eq!(cpu.regs.ax, 1);
    }
}
