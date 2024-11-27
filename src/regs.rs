use std::fmt::Display;

#[allow(unused)]
//struct Cpu {
//
//}

use::paste::paste;

pub struct Flags {
    bi: u16,
}

impl Default for Flags {
    fn default() -> Self {
        let mut f = Self { bi: Default::default() };
        f.bi |= 2;
        f
    }
}

impl Flags {
    pub fn clear_arith(&mut self) {
        self.clear_af();
        self.clear_cf();
        self.clear_sf();
        self.clear_zf();
        self.clear_of();
        self.clear_pf();
    }

    #[inline(always)]
    pub fn clear_cf(&mut self) {
        self.bi &= 0b1111111111111110;
    }
    #[inline(always)]
    pub fn set_cf(&mut self) {
        self.bi |= !0b1111111111111110;
    }
    #[inline(always)]
    pub fn cf(&self) -> bool {
        self.bi & !0b1111111111111110 > 0
    }

    #[inline(always)]
    pub fn clear_pf(&mut self) {
        self.bi &= 0b1111111111111011;
    }
    #[inline(always)]
    pub fn set_pf(&mut self) {
        self.bi |= !0b1111111111111011;
    }
    #[inline(always)]
    pub fn pf(&self) -> bool {
        self.bi & !0b1111111111111011 > 0
    }


    #[inline(always)]
    pub fn clear_af(&mut self) {
        self.bi &= 0b111111111101111;
    }
    #[inline(always)]
    pub fn set_af(&mut self) {
        self.bi |= !0b1111111111101111;
    }
    #[inline(always)]
    pub fn af(&self) -> bool {
        self.bi & !0b1111111111101111 > 0
    }


    #[inline(always)]
    pub fn clear_zf(&mut self) {
        self.bi &= 0b1111111110111111;
    }
    #[inline(always)]
    pub fn set_zf(&mut self) {
        self.bi |= !0b1111111110111111;
    }
    #[inline(always)]
    pub fn zf(&self) -> bool {
        self.bi & !0b1111111110111111 > 0
    }



    #[inline(always)]
    pub fn clear_sf(&mut self) {
        self.bi &= 0b1111111101111111;
    }
    #[inline(always)]
    pub fn set_sf(&mut self) {
        self.bi |= !0b1111111101111111;
    }
    #[inline(always)]
    pub fn sf(&self) -> bool {
        self.bi & !0b1111111101111111 > 0
    }


    #[inline(always)]
    pub fn clear_tf(&mut self) {
        self.bi &= 0b1111111011111111;
    }
    #[inline(always)]
    pub fn set_tf(&mut self) {
        self.bi |= !0b1111111011111111;
    }
    #[inline(always)]
    pub fn tf(&self) -> bool {
        self.bi & !0b1111111011111111 > 0
    }


    #[inline(always)]
    pub fn clear_if(&mut self) {
        self.bi &= 0b1111110111111111;
    }
    #[inline(always)]
    pub fn set_if(&mut self) {
        self.bi |= !0b1111110111111111;
    }
    #[inline(always)]
    pub fn i_f(&self) -> bool {
        self.bi & !0b1111110111111111 > 0
    }


    #[inline(always)]
    pub fn clear_df(&mut self) {
        self.bi &= 0b1111101111111111;
    }
    #[inline(always)]
    pub fn set_df(&mut self) {
        self.bi |= !0b1111101111111111;
    }
    #[inline(always)]
    pub fn df(&self) -> bool {
        self.bi & !0b1111101111111111 > 0
    }

    #[inline(always)]
    pub fn clear_of(&mut self) {
        self.bi &= 0b1111011111111111;
    }
    #[inline(always)]
    pub fn set_of(&mut self) {
        self.bi |= !0b1111011111111111;
    }
    #[inline(always)]
    pub fn of(&self) -> bool {
        self.bi & !0b1111011111111111 > 0
    }

}

impl Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PF: {}\nCF: {}\nOF: {}\nSF: {}\nAF: {}\nZF: {}",
        self.pf(), self.cf(), self.of(), self.sf(), self.af(), self.zf()
        )
    }
}

pub struct Registers {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub si: u16,
    pub di: u16,
    pub sp: u16,
    pub bp: u16,
    pub es: u32,
    pub ds: u32,
    pub cs: u32,
    pub ss: u32,
    pub flags: Flags
}


macro_rules! getsetreg {
    ($full:ident, $low:ident, $high:ident) => {
        paste!{
        #[inline(always)]
        pub fn [<set_ $full>] (&mut self, val: u16) {
            self.$full = val;
        }
        #[inline(always)]
        pub fn [<get_ $full>](&self) -> u16 {
            self.$full
        }
        #[inline(always)]
        pub fn [<get_ $low>](&self) -> u8 {
            (self.$full & 0xff) as u8
        }
        #[inline(always)]
        pub fn [<set_ $low>](&mut self, val:u8) {
            self.$full &= 0xff00;
            self.$full |= val as u16;
        }
        #[inline(always)]
        pub fn [<get_ $high>](&self) -> u8 {
            (self.$full >> 8) as u8
        }
        #[inline(always)]
        pub fn [<set_ $high>](&mut self, val:u8) {
            self.$full &= 0xff;
            self.$full |= (val as u16) << 8;
        } 
        }
    };
}

impl Registers {
    getsetreg!(ax, al, ah);
    getsetreg!(bx, bl, bh);
    getsetreg!(cx, cl, ch);
    getsetreg!(dx, dl, dh);
    #[inline(always)]
    pub fn get_si(&self) -> u16 {
        self.si
    }
    #[inline(always)]
    pub fn set_si(&mut self, val: u16) {
        self.si = val;
    } 
    #[inline(always)]
    pub fn get_di(&self) -> u16 {
        self.di
    }
    #[inline(always)]
    pub fn set_di(&mut self, val: u16) {
        self.di = val;
    } 
    #[inline(always)]
    pub fn get_bp(&self) -> u16 {
        self.bp
    }
    #[inline(always)]
    pub fn set_bp(&mut self, val: u16) {
        self.bp = val;
    } 

    #[inline(always)]
    pub fn get_sp(&self) -> u16 {
        self.sp
    }
    #[inline(always)]
    pub fn set_sp(&mut self, val: u16) {
        self.sp = val;
    } 

    #[inline(always)]
    pub fn get_ss(&self) -> u32 {
        self.ss << 4
    }
    #[inline(always)]
    pub fn set_ss(&mut self, val: u32) {
        assert!(val%16 == 0);
        self.ss = val >> 4;
    } 

    #[inline(always)]
    pub fn get_cs(&self) -> u32 {
        self.cs << 4
    }
    #[inline(always)]
    pub fn set_cs(&mut self, val: u32) {
        assert!(val%16 == 0);
        self.cs = val >> 4;
    } 

    #[inline(always)]
    pub fn get_ds(&self) -> u32 {
        self.ds << 4
    }
    #[inline(always)]
    pub fn set_ds(&mut self, val: u32) {
        assert!(val%16 == 0);
        self.ds = val >> 4;
    } 

    #[inline(always)]
    pub fn get_es(&self) -> u32 {
        self.es << 4
    }
    #[inline(always)]
    pub fn set_es(&mut self, val: u32) {
        assert!(val%16 == 0);
        self.es = val>>4;
    } 
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            ax: Default::default(),
            bx: Default::default(),
            cx: Default::default(),
            dx: Default::default(),
            si: Default::default(),
            sp: Default::default(),
            di: Default::default(),
            bp: Default::default(),
            ss: Default::default(),
            ds: Default::default(),
            es: Default::default(),
            cs: Default::default(),
            flags: Flags::default()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test1() {
        let mut regs = Registers::default();
        regs.set_ax(30000);
        assert_eq!(regs.get_ax(), 30000);
        assert_eq!(regs.get_al() as u16, 30000 & 0xff);
        assert_eq!(regs.get_ah() as u16, (30000 >> 8) );
        regs.set_ah(40);
        assert_eq!(regs.get_ah(), 40);
        assert_eq!(regs.get_al() as u16, 30000 & 0xff);
        regs.set_al(69);
        assert_eq!(regs.get_al(), 69);
        assert_eq!(regs.get_ah(), 40);

        regs.set_bx(30000);
        assert_eq!(regs.get_bx(), 30000);
        assert_eq!(regs.get_bl() as u16, 30000 & 0xff);
        assert_eq!(regs.get_bh() as u16, (30000 >> 8) );
        regs.set_bh(40);
        assert_eq!(regs.get_bh(), 40);
        assert_eq!(regs.get_bl() as u16, 30000 & 0xff);
        regs.set_bl(69);
        assert_eq!(regs.get_bl(), 69);
        assert_eq!(regs.get_bh(), 40);
    }

    #[test]
    pub fn test2() {
        let mut f:Flags = Flags::default();
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
}

