use cpu::Cpu;

#[allow(unused)]
mod regs;
#[allow(unused)]
mod cpu;
#[allow(unused)]
mod mem;


fn main() {
    let mut cpu = Cpu::init();
    cpu.load_code("/home/gg/Desktop/emulator/pp");

    cpu.mem.seek_to(cpu.code_addr(0) as u64);


    //cpu.regs.set_ax(110);

    while let Some(i) = cpu.fetch() {
        println!("{:?}", i);
        cpu.execute(&i);
    }

    println!("AX: {}", cpu.regs.get_ax());
    println!("{}", cpu.regs.flags);
}
