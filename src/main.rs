use cpu::Cpu;

#[allow(unused)]
mod cpu;
#[allow(unused)]
mod mem;
#[allow(unused)]
mod regs;

fn main() {
    let mut cpu = Cpu::init();
    //cpu.load_code("/home/gg/Desktop/emulator/pp");
    while let Some(i) = cpu.fetch() {
        println!("[{}] {:?}", cpu.regs.ip, i);
        cpu.execute(&i);
    }

    println!("AX: {}", cpu.regs.get_ax());
    println!("{}", cpu.regs.flags);
    println!("{:?}", 0i8.overflowing_add(1))
}
