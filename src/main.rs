use std::{env::args, process::exit};

use comfy_table::{Row, Table};
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
    let mut optable = comfy_table::Table::new();
    optable.set_header(
        Row::from(&["Option", "value"])
    ).add_row(
        Row::from(&["-f", "binary file"])
    );
    
    println!("{}", optable);
    exit(1);

}
fn main() {
    let mut cpu = Cpu::init();
    cpu.test_mode();
    let mut args = args();

    let mut file_found = false;

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
            }
        } else {
            break;
        }
    }

    if !file_found {
        print_usement();
    }

    while let Some(i) = cpu.fetch() {
        cpu.execute(&i);

        if cpu.halt {
            break;
        }
    }

    let mut regtable = Table::new();
    regtable.set_header(Row::from(&["Register", "Value"])).add_rows(
        vec![
            &["AX", &cpu.regs.ax.to_string()],
            &["BX", &cpu.regs.bx.to_string()],
            &["CX", &cpu.regs.cx.to_string()],
            &["DX", &cpu.regs.dx.to_string()],
            &["SI", &cpu.regs.si.to_string()],
            &["DI", &cpu.regs.di.to_string()],
            &["SP", &cpu.regs.sp.to_string()],
            &["BP", &cpu.regs.bp.to_string()],
        ]
    );
    println!("{}", regtable);

    let mut flagstable = Table::new();
    flagstable.set_header(&["Flag", "Value"]);
    flagstable.add_rows(vec![
        &["Parity",&cpu.regs.flags.pf().to_string()],
        &["Overflow",&cpu.regs.flags.of().to_string()],
        &["Sign",&cpu.regs.flags.sf().to_string()],
        &["Carry",&cpu.regs.flags.cf().to_string()],
        &["Zero",&cpu.regs.flags.zf().to_string()],
        &["Aux",&cpu.regs.flags.af().to_string()],
        &["Direction",&cpu.regs.flags.df().to_string()],
        &["Interrupt",&cpu.regs.flags.i_f().to_string()],
        &["Trap",&cpu.regs.flags.tf().to_string()],
    ]);
    println!("{}", flagstable)
}
