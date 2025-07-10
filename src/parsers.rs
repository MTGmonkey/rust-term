//! docs are more or less from [this document](https://real-world-systems.com/docs/ANSIcode.html)
//! this source file `parsers.rs` is licensed under the Mozilla Public License 2.0 (MPL2.0)
//! as parts of it are from [this](https://gitlab.com/davidbittner/ansi-parser) excellent crate

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit0, digit1};
use nom::combinator::{map, map_res, opt, value};
use nom::sequence::{delimited, preceded};
use nom::{IResult, Parser};

use crate::enums::{C0, EscapeSequence};

macro_rules! tag_parser {
    ($sig:ident, $tag:expr, $ret:expr) => {
        fn $sig(input: &str) -> IResult<&str, EscapeSequence> {
            value($ret, tag($tag)).parse(input)
        }
    };
}

macro_rules! one_ctlseq {
    ($sig:ident, $tag:expr, $def:expr, $ret:expr) => {
        fn $sig(input: &str) -> IResult<&str, EscapeSequence> {
            map(
                delimited(tag("["), |a| parse_maybe_int(a, $def), tag($tag)),
                |am| $ret(am),
            )
            .parse(input)
        }
    };
}

macro_rules! two_ctlseq {
    ($sig:ident, $tag:expr, $def0:expr, $def1:expr, $ret:expr) => {
        fn $sig(input: &str) -> IResult<&str, EscapeSequence> {
            map(
                (
                    tag("["),
                    |a| parse_maybe_int(a, $def0),
                    opt(tag(";")),
                    |b| parse_maybe_int(b, $def1),
                    tag($tag),
                ),
                |(_, a, _, b, _)| $ret(a, b),
            )
            .parse(input)
        }
    };
}

fn parse_u32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>()).parse(input)
}

fn parse_maybe_int(input: &str, default: u32) -> IResult<&str, u32> {
    map(digit0, |s: &str| s.parse::<u32>().unwrap_or(default)).parse(input)
}

fn combined(input: &str) -> IResult<&str, EscapeSequence> {
    alt((
        alt((
            ICH, CUU, CUD, CUF, CUB, CNL, CPL, CHA, CUP, CHT, ED, EL, IL, DL, EF, EA, DCH, SEM,
            CPR, SU, SD,
        )),
        alt((
            NP,
            PP,
            CTC,
            ECH,
            CVT,
            CBT,
            HPA,
            HPR,
            REP,
            DA,
            VPA,
            VPR,
            HVP,
            TBC,
            SM,
            MC,
            PageFormatSelect,
            RM,
            /*SGR,*/ DSR,
            DAQ,
        )),
        alt((
            DECLL, DECSTBM, DECSTRM, DECSLPP, /*DECSHTS,*/ /*DECSVTS,*/ DECSHORP,
            /*DECREQTPARM,*/ DECTST, DECVERP, DECTTC, /*DECPRO,*/ DECKEYS,
            /*DELETE,*/ SL, SR, GSM, GSS, FNT, TSS, JFY,
        )),
        SPI,
        QUAD,
    ))
    .parse(input)
}

pub fn parse_escape(input: &str) -> IResult<&str, EscapeSequence> {
    preceded(tag("\u{1b}"), combined).parse(input)
}

one_ctlseq!(ICH, "@", 1, EscapeSequence::ICH);
one_ctlseq!(CUU, "A", 1, EscapeSequence::CUU);
one_ctlseq!(CUD, "B", 1, EscapeSequence::CUD);
one_ctlseq!(CUF, "C", 1, EscapeSequence::CUF);
one_ctlseq!(CUB, "D", 1, EscapeSequence::CUB);
one_ctlseq!(CNL, "E", 1, EscapeSequence::CNL);
one_ctlseq!(CPL, "F", 1, EscapeSequence::CPL);
one_ctlseq!(CHA, "G", 1, EscapeSequence::CHA);
two_ctlseq!(CUP, "H", 1, 1, EscapeSequence::CUP);
one_ctlseq!(CHT, "I", 1, EscapeSequence::CHT);
one_ctlseq!(ED, "J", 0, EscapeSequence::ED);
one_ctlseq!(EL, "K", 0, EscapeSequence::EL);
one_ctlseq!(IL, "L", 0, EscapeSequence::IL);
one_ctlseq!(DL, "M", 0, EscapeSequence::DL);
one_ctlseq!(EF, "N", 0, EscapeSequence::EF);
one_ctlseq!(EA, "O", 0, EscapeSequence::EA);
one_ctlseq!(DCH, "P", 1, EscapeSequence::DCH);
one_ctlseq!(SEM, "Q", 0, EscapeSequence::SEM);
tag_parser!(CPR, "[R", EscapeSequence::CPR);
one_ctlseq!(SU, "S", 1, EscapeSequence::SU);
one_ctlseq!(SD, "T", 1, EscapeSequence::SD);
one_ctlseq!(NP, "U", 1, EscapeSequence::NP);
one_ctlseq!(PP, "V", 1, EscapeSequence::PP);
one_ctlseq!(CTC, "W", 0, EscapeSequence::CTC);
one_ctlseq!(ECH, "X", 1, EscapeSequence::ECH);
one_ctlseq!(CVT, "Y", 1, EscapeSequence::CVT);
one_ctlseq!(CBT, "Z", 1, EscapeSequence::CBT);
one_ctlseq!(HPA, "`", 0, EscapeSequence::HPA);
one_ctlseq!(HPR, "a", 0, EscapeSequence::HPR);
one_ctlseq!(REP, "b", 1, EscapeSequence::REP);
tag_parser!(DA, "[c", EscapeSequence::DA);
one_ctlseq!(VPA, "d", 0, EscapeSequence::VPA);
one_ctlseq!(VPR, "e", 0, EscapeSequence::VPR);
two_ctlseq!(HVP, "f", 1, 1, EscapeSequence::HVP);
one_ctlseq!(TBC, "g", 0, EscapeSequence::TBC);
one_ctlseq!(SM, "h", 0, EscapeSequence::SM);
one_ctlseq!(MC, "i", 0, EscapeSequence::MC);
one_ctlseq!(PageFormatSelect, "j", 0, EscapeSequence::PageFormatSelect);
tag_parser!(RM, "[l", EscapeSequence::RM);
// TODO SGR takes 5 parameters
one_ctlseq!(DSR, "n", 0, EscapeSequence::DSR);
one_ctlseq!(DAQ, "o", 0, EscapeSequence::DAQ);
tag_parser!(DECLL, "[q", EscapeSequence::DECLL);
two_ctlseq!(DECSTBM, "r", 1, 1, EscapeSequence::DECSTBM);
two_ctlseq!(DECSTRM, "s", 1, 1, EscapeSequence::DECSTRM);
one_ctlseq!(DECSLPP, "t", 66, EscapeSequence::DECSLPP);
// TODO DECSHTS takes many parameters
// TODO DECSVTS takes many parameters
one_ctlseq!(DECSHORP, "w", 0, EscapeSequence::DECSHORP);
// TODO DECREQTPARM takes many parameters
two_ctlseq!(DECTST, "y", 2, 1, EscapeSequence::DECTST);
one_ctlseq!(DECVERP, "z", 0, EscapeSequence::DECVERP);
one_ctlseq!(DECTTC, "!", 0, EscapeSequence::DECTTC);
// TODO DECPRO takes many parameters
one_ctlseq!(DECKEYS, "~", 0, EscapeSequence::DECKEYS);
// TODO DELETE needs reading into, what hex code do I need
one_ctlseq!(SL, " @", 1, EscapeSequence::SL);
one_ctlseq!(SR, " A", 1, EscapeSequence::SR);
two_ctlseq!(GSM, " B", 100, 100, EscapeSequence::GSM);
one_ctlseq!(GSS, " C", 720, EscapeSequence::GSS);
two_ctlseq!(FNT, " D", 0, 1, EscapeSequence::FNT);
one_ctlseq!(TSS, " E", 720, EscapeSequence::TSS);
one_ctlseq!(JFY, " F", 0, EscapeSequence::JFY);
two_ctlseq!(SPI, " G", 720, 720, EscapeSequence::SPI);
one_ctlseq!(QUAD, " H", 0, EscapeSequence::QUAD);
