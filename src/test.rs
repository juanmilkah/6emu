use crate::{
    cpu::{self, Cpu, Instruction, Opcode, Operand},
    regs::{Flags, Registers},
};

#[test]
fn flow1() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![
        184, 0, 0, 185, 1, 0, 57, 200, 119, 2, 235, 3, 184, 69, 0,
    ]);
    cpu.fire();
    assert_eq!(cpu.regs.ax, 0);
}

#[test]
fn loopy() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![
        185, 20, 0, 49, 192, 137, 4, 64, 255, 4, 226, 251, 3, 4,
    ]);
    cpu.fire();
    assert_eq!(cpu.regs.ax, 40);
}

#[test]
fn flow0() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![
        184, 1, 0, 185, 0, 0, 57, 200, 119, 2, 235, 3, 184, 69, 0,
    ]);
    cpu.fire();
    assert_eq!(cpu.regs.ax, 69);
}

#[test]
fn stack() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![
        184, 70, 0, 185, 45, 0, 186, 89, 0, 187, 132, 3, 83, 82, 81, 80, 91, 90, 89, 88,
    ]);
    cpu.fire();
    assert_eq!(cpu.regs.ax, 900);
    assert_eq!(cpu.regs.cx, 89);
    assert_eq!(cpu.regs.dx, 45);
    assert_eq!(cpu.regs.bx, 70);
}

#[test]
fn memstuff() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![198, 4, 0, 198, 68, 1, 1, 139, 4]);
    cpu.fire();
    assert_eq!(cpu.regs.get_ax(), 256);
}

#[test]
fn addmem16() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![199, 4, 0, 0, 131, 192, 70, 1, 4, 139, 4]);
    cpu.fire();
    assert_eq!(cpu.regs.get_ax(), 70);
}

#[test]
fn subreg8() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![
        128, 236, 67, 40, 224, 40, 197, 40, 233, 40, 207, 40, 251, 40, 222, 40, 242,
    ]);
    cpu.fire();
    assert!(cpu.regs.get_dl() == 67);
}

#[test]
fn addreg8() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![
        128, 196, 67, 0, 224, 0, 197, 0, 233, 0, 207, 0, 251, 0, 222, 0, 242,
    ]);
    cpu.fire();
    assert!(cpu.regs.get_dl() == 67);
}

#[test]
fn addreg16() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![131, 192, 67, 1, 200, 1, 195, 1, 218]);
    cpu.fire();
    assert!(cpu.regs.dx == 67);
}

#[test]
fn addregimm16() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![
        131, 192, 67, 129, 193, 207, 7, 131, 195, 120, 129, 194, 0, 3,
    ]);
    cpu.fire();
    assert!(cpu.regs.ax == 67);
    assert!(cpu.regs.cx == 1999);
    assert!(cpu.regs.bx == 120);
    assert!(cpu.regs.dx == 768);
}

#[test]
fn addregimm8() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![
        4, 1, 128, 196, 1, 128, 193, 1, 128, 197, 1, 128, 195, 1, 128, 199, 1, 128, 194, 1, 128,
        198, 1,
    ]);
    cpu.fire();
    assert!(cpu.regs.ax == 0x101);
    assert!(cpu.regs.bx == 0x101);
    assert!(cpu.regs.cx == 0x101);
    assert!(cpu.regs.dx == 0x101);
}

#[test]
fn test_mode() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    cpu.load_code_vec(&vec![140, 209, 137, 224]);
    cpu.fire();
    assert_eq!(cpu.regs.sp, 4095);
    assert_eq!(cpu.regs.get_ss(), 4096)
}

#[test]
pub fn test1() {
    let mut regs = Registers::default();
    regs.set_ax(30000);
    assert_eq!(regs.get_ax(), 30000);
    assert_eq!(regs.get_al() as u16, 30000 & 0xff);
    assert_eq!(regs.get_ah() as u16, (30000 >> 8));
    regs.set_ah(40);
    assert_eq!(regs.get_ah(), 40);
    assert_eq!(regs.get_al() as u16, 30000 & 0xff);
    regs.set_al(69);
    assert_eq!(regs.get_al(), 69);
    assert_eq!(regs.get_ah(), 40);

    regs.set_bx(30000);
    assert_eq!(regs.get_bx(), 30000);
    assert_eq!(regs.get_bl() as u16, 30000 & 0xff);
    assert_eq!(regs.get_bh() as u16, (30000 >> 8));
    regs.set_bh(40);
    assert_eq!(regs.get_bh(), 40);
    assert_eq!(regs.get_bl() as u16, 30000 & 0xff);
    regs.set_bl(69);
    assert_eq!(regs.get_bl(), 69);
    assert_eq!(regs.get_bh(), 40);
}

#[test]
pub fn test2() {
    let mut f: Flags = Flags::default();
    assert_eq!(f.bi, 2);
    f.set_cf();
    assert!(f.bi == 3);
    assert!(f.cf());
    f.set_af();
    assert!(f.af());
    assert!(f.cf());
    f.set_df();
    assert!(f.af());
    assert!(f.cf());
    assert!(f.df());
    f.set_if();
    assert!(f.af());
    assert!(f.cf());
    assert!(f.df());
    assert!(f.i_f());
    f.set_of();
    assert!(f.af());
    assert!(f.cf());
    assert!(f.df());
    assert!(f.i_f());
    assert!(f.of());
    f.set_pf();
    assert!(f.af());
    assert!(f.cf());
    assert!(f.df());
    assert!(f.i_f());
    assert!(f.of());
    assert!(f.pf());
    f.set_sf();
    assert!(f.af());
    assert!(f.cf());
    assert!(f.df());
    assert!(f.i_f());
    assert!(f.of());
    assert!(f.pf());
    assert!(f.sf());
    f.set_tf();
    assert!(f.af());
    assert!(f.cf());
    assert!(f.df());
    assert!(f.i_f());
    assert!(f.of());
    assert!(f.pf());
    assert!(f.sf());
    assert!(f.tf());
    f.set_zf();
    assert!(f.af());
    assert!(f.cf());
    assert!(f.df());
    assert!(f.i_f());
    assert!(f.of());
    assert!(f.pf());
    assert!(f.sf());
    assert!(f.sf());
    assert!(f.zf());

    f.clear_cf();
    assert!(!f.cf());
    f.clear_af();
    assert!(!f.af());
    assert!(!f.cf());
    f.clear_df();
    assert!(!f.af());
    assert!(!f.cf());
    assert!(!f.df());
    f.clear_if();
    assert!(!f.af());
    assert!(!f.cf());
    assert!(!f.df());
    assert!(!f.i_f());
    f.clear_of();
    assert!(!f.af());
    assert!(!f.cf());
    assert!(!f.df());
    assert!(!f.i_f());
    assert!(!f.of());
    f.clear_pf();
    assert!(!f.af());
    assert!(!f.cf());
    assert!(!f.df());
    assert!(!f.i_f());
    assert!(!f.of());
    assert!(!f.pf());
    f.clear_sf();
    assert!(!f.af());
    assert!(!f.cf());
    assert!(!f.df());
    assert!(!f.i_f());
    assert!(!f.of());
    assert!(!f.pf());
    assert!(!f.sf());
    f.clear_tf();
    assert!(!f.af());
    assert!(!f.cf());
    assert!(!f.df());
    assert!(!f.i_f());
    assert!(!f.of());
    assert!(!f.pf());
    assert!(!f.tf());
    assert!(!f.sf());
    f.clear_zf();
    assert!(!f.af());
    assert!(!f.cf());
    assert!(!f.df());
    assert!(!f.i_f());
    assert!(!f.of());
    assert!(!f.pf());
    assert!(!f.sf());
    assert!(!f.sf());
    assert!(!f.zf());
}

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
    cpu.test_mode();
    cpu.load_code_vec(&[
        0x2, 0x2e, 0x50, 0x0, 0x0, 0xc0, 0x0, 0xc9, 0x0, 0xe4, 0x0, 0xdb, 0x0, 0xff, 0x0, 0xed,
        0x0, 0xc9, 0x1, 0xc0, 0x1, 0xdb, 0x1, 0xc9, 0x1, 0xd2, 0x1, 0x6, 0x5a, 0x0, 0x3, 0x4, 0x1,
        0xf6, 0x1, 0xff, 0x1, 0xed, 0x1, 0xe4, 0x1, 0x4, 0x1, 0x1d,
    ]);
    cpu.mem.seek_to(cpu.code_addr(0) as u64);

    //cpu.regs.set_cs(0);
    cpu.regs.set_ds(0);
    cpu.regs.set_ss(0);
    cpu.regs.set_es(0);

    let b1 = cpu.fetch().unwrap();
    assert!(b1.opcode() == Opcode::Add);
    //assert_eq!(b1.operands(), (Operand::Reg8(5), Operand::Mem8(80)));

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
    //assert_eq!(b1.operands(), (Operand::Mem16(90), Operand::Reg16(0)));

    let b1 = cpu.fetch().unwrap();
    assert!(b1.opcode() == Opcode::Add);
    //assert_eq!(b1.operands(), (Operand::Reg16(0), Operand::Mem16(0)));

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
    //assert_eq!(b1.operands(), (Operand::Mem16(90), Operand::Reg16(0)));

    cpu.regs.set_di(90);
    let b1 = cpu.fetch().unwrap();
    assert!(b1.opcode() == Opcode::Add);
    //assert_eq!(b1.operands(), (Operand::Mem16(90), Operand::Reg16(3)));
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
