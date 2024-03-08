use paste::paste;

use crate::{
    asm::{LocalTryInto, Mask},
    instruction,
    prelude::*,
    ParseError,
    ToThumb,
};

instruction!(
    size u32; A5_17 contains
    Strex : {
        imm as u8 : u8          : 0 -> 7,
        rd  as u8 : Register    : 8 -> 11 try_into,
        rt  as u8 : Register    : 12 -> 15 try_into,
        rn  as u8 : Register    : 16 -> 19 try_into
    },
    Ldrex : {
        imm as u8 : u8          : 0 -> 7,
        rt  as u8 : Register    : 12 -> 15 try_into,
        rn  as u8 : Register    : 16 -> 19 try_into
    },
    Strd : {
        imm as u8   : u8          : 0 -> 7,
        rt2  as u8  : Register    : 8 -> 11 try_into,
        rt  as u8   : Register    : 12 -> 15 try_into,
        rn  as u8   : Register    : 16 -> 19 try_into,
        w   as u8   : bool        : 21 -> 21 local_try_into,
        u   as u8   : bool        : 23 -> 23 local_try_into,
        p   as u8   : bool        : 24 -> 24 local_try_into
    },
    Ldrd : {
        imm as u8   : u8          : 0 -> 7,
        rt2  as u8  : Register    : 8 -> 11 try_into,
        rt  as u8   : Register    : 12 -> 15 try_into,
        rn  as u8   : Register    : 16 -> 19 try_into,
        w   as u8   : bool        : 21 -> 21 local_try_into,
        u   as u8   : bool        : 23 -> 23 local_try_into,
        p   as u8   : bool        : 24 -> 24 local_try_into
    },
    Strexb : {
        rd  as u8   : Register    : 0 -> 3 try_into,
        rt  as u8   : Register    : 12 -> 15 try_into,
        rn  as u8   : Register    : 16 -> 19 try_into
    },
    Strexh : {
        rd  as u8   : Register    : 0 -> 3 try_into,
        rt  as u8   : Register    : 12 -> 15 try_into,
        rn  as u8   : Register    : 16 -> 19 try_into
    },
    Tbb : {
        rm as u8    : Register    : 0 -> 3 try_into,
        // Denotes if it is a halfword or a full word
        h  as u8    : bool        : 4 -> 4 local_try_into,
        rn as u8    : Register    : 16 -> 19 try_into
    },
    Ldrexb : {
        rt as u8    : Register    : 12 -> 15 try_into,
        rn as u8    : Register    : 16 -> 19 try_into
    },
    Ldrexh : {
        rt as u8    : Register    : 12 -> 15 try_into,
        rn as u8    : Register    : 16 -> 19 try_into
    }
);

impl Parse for A5_17 {
    type Target = Self;

    fn parse<T: Stream>(iter: &mut T) -> Result<Self::Target, ParseError>
    where
        Self: Sized,
    {
        let word: u32 = match iter.peek::<1>() {
            Some(val) => Ok(val),
            None => Err(ParseError::IncompleteProgram),
        }?;
        let op3 = word.mask::<4, 7>();
        let op2 = word.mask::<20, 21>();
        let op1 = word.mask::<23, 24>();

        if op1 == 00 {
            match op2 {
                0 => return Ok(Self::Strex(Strex::parse(iter)?)),
                1 => return Ok(Self::Ldrex(Ldrex::parse(iter)?)),
                _ => {}
            }
        }
        if (op1 >> 1 == 0 && op2 == 2) || (op1 >> 1 == 1 && op2 & 0b1 == 0) {
            return Ok(Self::Strd(Strd::parse(iter)?));
        }
        if (op1 >> 1 == 0 && op2 == 3) || (op1 >> 1 == 1 && op2 & 0b1 == 1) {
            return Ok(Self::Ldrd(Ldrd::parse(iter)?));
        }
        if op1 != 0b01 {
            return Err(ParseError::Invalid32Bit("A5_17"));
        }
        match (op2, op3) {
            (0, 0b100) => Ok(Self::Strexb(Strexb::parse(iter)?)),
            (0, 0b101) => Ok(Self::Strexh(Strexh::parse(iter)?)),
            (1, 0) | (1, 1) => Ok(Self::Tbb(Tbb::parse(iter)?)),
            (1, 0b100) => Ok(Self::Ldrexb(Ldrexb::parse(iter)?)),
            (1, 0b101) => Ok(Self::Ldrexh(Ldrexh::parse(iter)?)),
            _ => Err(ParseError::Invalid32Bit("A5_17")),
        }
    }
}
impl ToThumb for A5_17 {
    fn encoding_specific_operations(self) -> thumb::Thumb {
        match self {
            Self::Strex(el) => {
                let imm = (el.imm as u32) << 2;
                thumb::Strex::builder()
                    .set_rd(el.rd)
                    .set_rt(el.rt)
                    .set_rn(el.rn)
                    .set_imm(Some(imm))
                    .complete()
                    .into()
            }
            Self::Ldrex(el) => {
                let imm = (el.imm as u32) << 2;
                thumb::Ldrex::builder()
                    .set_rt(el.rt)
                    .set_rn(el.rn)
                    .set_imm(imm)
                    .complete()
                    .into()
            }
            Self::Strd(el) => thumb::StrdImmediate::builder()
                .set_w(Some(el.w))
                .set_rt(el.rt)
                .set_index(Some(el.p))
                .set_rn(el.rn)
                .set_add(el.u)
                .set_rt2(el.rt2)
                .set_imm(Some(el.imm as u32))
                .complete()
                .into(),
            Self::Ldrd(el) => thumb::LdrdImmediate::builder()
                .set_w(Some(el.w))
                .set_add(Some(el.u))
                .set_rt(el.rt)
                .set_rn(el.rn)
                .set_rt2(el.rt2)
                .set_index(Some(el.p))
                .set_imm((el.imm as u32) << 2)
                .complete()
                .into(),
            Self::Strexb(el) => thumb::Strexb::builder()
                .set_rd(el.rd)
                .set_rt(el.rt)
                .set_rn(el.rn)
                .complete()
                .into(),
            Self::Strexh(el) => thumb::Strexh::builder()
                .set_rd(el.rd)
                .set_rt(el.rt)
                .set_rn(el.rn)
                .complete()
                .into(),
            Self::Tbb(el) => thumb::Tb::builder()
                .set_is_tbh(Some(el.h))
                .set_rn(el.rn)
                .set_rm(el.rm)
                .complete()
                .into(),
            Self::Ldrexb(el) => thumb::Ldrexb::builder()
                .set_rt(el.rt)
                .set_rn(el.rn)
                .complete()
                .into(),
            Self::Ldrexh(el) => thumb::Ldrexh::builder()
                .set_rt(el.rt)
                .set_rn(el.rn)
                .complete()
                .into(),
        }
    }
}
