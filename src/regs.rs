use std::fmt::Display;

#[allow(unused)]
//struct Cpu {
//
//}
use ::paste::paste;

pub struct Flags {
    pub bi: u16,
}

impl Default for Flags {
    fn default() -> Self {
        let mut f = Self {
            bi: Default::default(),
        };
        f.bi |= 2;
        f
    }
}

impl Flags {
    pub fn clear_arith(&mut self) {
        self.clear_cf();
        self.clear_af();
        self.clear_sf();
        self.clear_zf();
        self.clear_of();
        self.clear_pf();
    }

    pub fn to_u16(&self) -> u16 {
        self.bi
    }

    pub fn set_from_u16(&mut self, val: u16) {
        self.bi = val
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
        write!(
            f,
            "PF: {}\nCF: {}\nOF: {}\nSF: {}\nAF: {}\nZF: {}",
            self.pf(),
            self.cf(),
            self.of(),
            self.sf(),
            self.af(),
            self.zf()
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

    pub es: u16,
    pub ds: u16,
    pub cs: u16,
    pub ss: u16,

    pub flags: Flags,
    pub ip: u16,
}

macro_rules! getsetreg {
    ($full:ident, $low:ident, $high:ident) => {
        paste! {
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
        (self.ss as u32) << 4
    }
    #[inline(always)]
    pub fn set_ss(&mut self, val: u32) {
        assert!(val % 16 == 0);
        self.ss = (val >> 4) as u16;
    }

    #[inline(always)]
    pub fn get_cs(&self) -> u32 {
        (self.cs as u32) << 4
    }
    #[inline(always)]
    pub fn set_cs(&mut self, val: u32) {
        assert!(val % 16 == 0);
        self.cs = (val >> 4) as u16;
    }

    #[inline(always)]
    pub fn get_ds(&self) -> u32 {
        (self.ds as u32) << 4
    }
    #[inline(always)]
    pub fn set_ds(&mut self, val: u32) {
        assert!(val % 16 == 0);
        self.ds = (val >> 4) as u16;
    }

    #[inline(always)]
    pub fn get_es(&self) -> u32 {
        (self.es as u32) << 4
    }
    #[inline(always)]
    pub fn set_es(&mut self, val: u32) {
        assert!(val % 16 == 0);
        self.es = (val >> 4) as u16;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            ip: Default::default(),
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
            flags: Flags::default(),
        }
    }
}
