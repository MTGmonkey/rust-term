//! docs are more or less from [this document](https://real-world-systems.com/docs/ANSIcode.html)
//! this source file `enums.rs` is licensed under the Mozilla Public License 2.0 (MPL2.0)
//! as parts of it are from [this](https://gitlab.com/davidbittner/ansi-parser) excellent crate

#![allow(dead_code)]

use crate::parsers::parse_escape;
use heapless::Vec;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Text(&'a str),
    C0(C0),
    EscapeSequence(EscapeSequence),
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Token::*;
        match self {
            Text(txt) => write!(f, "{}", txt),
            C0(c0) => write!(f, "{:?}", c0),
            EscapeSequence(escape_sequence) => write!(f, "{:?}", escape_sequence),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum C0 {
    NUL,
    SOH,
    STX,
    ETX,
    EOT,
    ENQ,
    ACK,
    BEL,
    BS,
    HT,
    LF,
    VT,
    FF,
    CR,
    SO,
    SI,
    DLE,
    DC1,
    DC2,
    DC3,
    DC4,
    NAK,
    SYN,
    ETB,
    CAN,
    EM,
    SUB,
    ESC,
    FS,
    GS,
    RS,
    US,
    SP,
    DEL,
}

// TODO figure out defaults
#[derive(Debug, PartialEq, Clone)]
pub enum EscapeSequence {
    ICH(u32),              // [#@ def 1 Insert CHaracter
    CUU(u32),              // [#A def 1 CUrsor Up
    CUD(u32),              // [#B def 1 CUrsor Down
    CUF(u32),              // [#C def 1 CUrsor Forward
    CUB(u32),              // [#D def 1 CUrsor Backward
    CNL(u32),              // [#E def 1 Cursor to Next Line
    CPL(u32),              // [#F def 1 Cursor to Previous Line
    CHA(u32),              // [#G def 1 Cursor Horizontal position Absolute
    CUP(u32, u32),         // [#;#H def 1;1 CUrsor Position
    CHT(u32),              // [#I def 1 Cursor Horizontal Tabulation
    ED(u32),               // [#J def 0 Erase in Display
    EL(u32),               // [#K def 0 Erase in Line
    IL(u32),               // [#L def 0 Insert Line, current line moves down
    DL(u32),               // [#M def 0 Delete Line, lines below current move up
    EF(u32),               // [#N def 0 Erase in Field
    EA(u32),               // [#O def 0 Erase in qualified Area
    DCH(u32),              // [#P def 1 Delete CHaracter
    SEM(u32),              // [#Q def 0 Set Editing extent Mode
    CPR,                   // [R        Cursor Position Report
    SU(u32),               // [#S def 1 Scroll Up
    SD(u32),               // [#T def 1 Scroll Down
    NP(u32),               // [#U def 1 Next Page
    PP(u32),               // [#V def 1 Previous Page
    CTC(u32),              // [#W def 0 Cursor Tabulation Control
    ECH(u32),              // [#X def 1 Erase CHaracter
    CVT(u32),              // [#Y def 1 Cursor Vertical Tab
    CBT(u32),              // [#Z def 1 Cursor Back Tab
    HPA(u32),              // [#` def 0 Horizontal Position Absolute
    HPR(u32),              // [#a def 0 Horizontal Position Relative
    REP(u32),              // [#b def 1 REPeat previous displayable character
    DA,                    // [c        Device Attributes
    VPA(u32),              // [#d def 0 Vertical Position Absolute
    VPR(u32),              // [#e def 0 Vertical Position Relative
    HVP(u32, u32),         // [#;#f def 0;0 Horizontal and Vertical Position
    TBC(u32),              // [#g def 0 TaBulation Clear
    SM(u32),               // [#h def 0 Set Mode
    MC(u32),               // [#i def 0 Media Copy
    PageFormatSelect(u32), // [#j def 0
    RM,                    // [l       Reset Mode
    SGR(Vec<u8, 5>),       // [#;#;#;#;#l def 0 Set Graphics Rendition
    DSR(u32),              // [#n def 0 Device Status Report
    DAQ(u32),              // [#o def 0 Define Area Qualification starting at current position
    DECLL,                 // [q UNIMPLEMENTED many params
    DECSTBM(u32, u32),     // [#;#r def 1;1 top and bottom margins
    DECSTRM(u32, u32),     // [#;#s def 1;1 left and right margins
    DECSLPP(u32),          // [#t def 66 physical lines per page
    DECSHTS,               // [u        UNIMPLEMENTED many params
    DECSVTS,               // [v        UNIMPLEMENTED many params
    DECSHORP(u32),         // [#w def 0 set horizontal pitch on LAxxx printers
    DECREQTPARM,           // [x UNIMPLEMENTED many params
    DECTST(u32, u32),      // [#;#y def 2;1 invoke confidence test
    DECVERP(u32),          // [#z def 0 set vertical -pitch on LA100
    DECTTC(u32),           // [#| def 0 transmit termination character
    DECPRO,                // [} UNIMPLEMENTED many params
    DECKEYS(u32),          // [#~ def 0 sent by special function keys
    DELETE,                // [DELETE   always ignored
    SL(u32),               // [# @ def 1 Scroll Left
    SR(u32),               // [# A def 1 Scroll Right
    GSM(u32, u32),         // [#;# B def 100;100 Graphic Size Modification
    GSS(u32),              // [# C def 720 Graphic Size Selection
    FNT(u32, u32),         // [#;# D def 0;1 FoNT selection
    TSS(u32),              // [# E def 720 Thin Space Specification
    JFY(u32),              // [# F def 0 JustiFY
    SPI(u32, u32),         // [#;# G def 720;720 SPacing Increment
    QUAD(u32),             // [# H def 0 do QUADding on current line of text
}

pub trait AnsiParser {
    fn ansi_parse(&self) -> AnsiParseIterator<'_>;
}

impl AnsiParser for str {
    fn ansi_parse(&self) -> AnsiParseIterator<'_> {
        AnsiParseIterator { dat: &self }
    }
}

impl AnsiParser for String {
    fn ansi_parse(&self) -> AnsiParseIterator<'_> {
        AnsiParseIterator { dat: self }
    }
}

#[derive(Debug)]
pub struct AnsiParseIterator<'a> {
    dat: &'a str,
}

impl<'a> Iterator for AnsiParseIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.dat.is_empty() {
            return None;
        }

        let (head, tail) = head(self.dat);
        let dat = self.dat;
        self.dat = tail;

        // bound checked
        use C0::*;
        Some(match head {
            "\u{00}" => Token::C0(NUL),
            "\u{01}" => Token::C0(SOH),
            "\u{02}" => Token::C0(STX),
            "\u{03}" => Token::C0(ETX),
            "\u{04}" => Token::C0(EOT),
            "\u{05}" => Token::C0(ENQ),
            "\u{06}" => Token::C0(ACK),
            "\u{07}" => Token::C0(BEL),
            "\u{08}" => Token::C0(BS),
            "\u{09}" => Token::C0(HT),
            "\u{0a}" => Token::C0(LF),
            "\u{0b}" => Token::C0(VT),
            "\u{0c}" => Token::C0(FF),
            "\u{0d}" => Token::C0(CR),
            "\u{0e}" => Token::C0(SO),
            "\u{0f}" => Token::C0(SI),
            "\u{10}" => Token::C0(DLE),
            "\u{11}" => Token::C0(DC1),
            "\u{12}" => Token::C0(DC2),
            "\u{13}" => Token::C0(DC3),
            "\u{14}" => Token::C0(DC4),
            "\u{15}" => Token::C0(NAK),
            "\u{16}" => Token::C0(SYN),
            "\u{17}" => Token::C0(ETB),
            "\u{18}" => Token::C0(CAN),
            "\u{19}" => Token::C0(EM),
            "\u{1a}" => Token::C0(SUB),
            "\u{1b}" => {
                if let Ok(ret) = parse_escape(dat) {
                    println!("^[ successfully parsed");
                    self.dat = ret.0.clone();
                    Token::EscapeSequence(ret.1)
                } else {
                    println!("^[ unimplemented");
                    Token::C0(ESC)
                }
            }
            "\u{1c}" => Token::C0(FS),
            "\u{1d}" => Token::C0(GS),
            "\u{1e}" => Token::C0(RS),
            "\u{1f}" => Token::C0(US),
            "\u{20}" => Token::C0(SP),
            "\u{7F}" => Token::C0(DEL),
            txt => Token::Text(txt),
        })
    }
}

fn head(string: &str) -> (&str, &str) {
    for i in 1..5 {
        let r = string.get(0..i);
        match r {
            Some(val) => return (val, &string[i..]),
            None => (),
        }
    }
    panic!();
}
