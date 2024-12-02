use std::{env::args,process::exit};

use cpu::Cpu;

#[allow(unused)]
mod cpu;
#[allow(unused)]
mod mem;
#[allow(unused)]
mod regs;

#[cfg(test)]
mod test;

fn print_usement() {
    println!("Usage: ./app options");

    println!("   -f binary file");
    
    exit(1);
}

fn exec_dump_state(cpu: &mut Cpu) {
    while let Some(i) = cpu.fetch() {
        cpu.execute(&i);

        if cpu.halt {
            break;
        }
    }
    println!("{{");
        println!("\"registers\":{{");
            println!("\"AX\":{},", cpu.regs.ax);
            println!("\"BX\":{},", cpu.regs.bx);
            println!("\"CX\":{},", cpu.regs.cx);
            println!("\"DX\":{},", cpu.regs.dx);
            println!("\"SI\":{},", cpu.regs.si);
            println!("\"DI\":{},", cpu.regs.di);
            println!("\"SP\":{},", cpu.regs.sp);
            println!("\"BP\":{}", cpu.regs.bp);
        println!("}},");
            println!("\"flags\": {{");
            println!("\"Parity\":{},",cpu.regs.flags.pf());
            println!("\"Overflow\":{},",&cpu.regs.flags.of());
            println!("\"Sign\":{},",&cpu.regs.flags.sf());
            println!("\"Carry\":{},",&cpu.regs.flags.cf());
            println!("\"Zero\":{},",&cpu.regs.flags.zf());
            println!("\"Aux\":{},",&cpu.regs.flags.af());
            println!("\"Direction\":{},",&cpu.regs.flags.df());
            println!("\"Interrupt\":{},",&cpu.regs.flags.i_f());
            println!("\"Trap\":{}",&cpu.regs.flags.tf());
        println!("}} }}");
}

fn main() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    let mut args = args();

    let mut file_found = false;

    let mut load_from_stdin = false;

    loop {
        if let Some(arg) = args.next() {
            if arg == "-f" {
                if let Some(name) = args.next() {
                    cpu.load_code(&name);
                    file_found = true;
                } else {
                    print_usement();
                    exit(1)
                }
            } else if arg == "--stdin" {
                cpu.load_code_stdin();
                load_from_stdin = true
            }
        } else {
            break;
        }
    }

    if !file_found && !load_from_stdin {
        print_usement();
    }

    exec_dump_state(&mut cpu);

}
