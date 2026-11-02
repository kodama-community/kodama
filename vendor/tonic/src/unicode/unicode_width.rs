//! Zero-width marks and East Asian full-width character classification,
//! mirroring the Unicode width tables of the MoonBit backend.

fn is_between(code: u32, lo: u32, hi: u32) -> bool {
    lo <= code && code <= hi
}
pub(crate) fn is_zero_width_mark(code: u32) -> bool {
    zero_width_marks_basic(code)
        || zero_width_marks_indic_a(code)
        || zero_width_marks_indic_b(code)
        || zero_width_marks_indic_c(code)
        || zero_width_marks_myanmar(code)
        || zero_width_marks_balinese(code)
        || zero_width_marks_cjk(code)
        || zero_width_marks_aaa(code)
        || zero_width_marks_smp_a(code)
        || zero_width_marks_smp_b(code)
        || zero_width_marks_smp_c(code)
        || zero_width_marks_smp_d(code)
}
fn zero_width_marks_basic(code: u32) -> bool {
    code <= 0x1F
        || code == 0x7F
        || is_between(code, 0x80, 0x9F)
        || is_between(code, 0x0300, 0x036F)
        || is_between(code, 0x0483, 0x0489)
        || is_between(code, 0x0591, 0x05BD)
        || code == 0x05BF
        || is_between(code, 0x05C1, 0x05C2)
        || is_between(code, 0x05C4, 0x05C5)
        || code == 0x05C7
        || is_between(code, 0x0610, 0x061A)
        || is_between(code, 0x064B, 0x065F)
        || code == 0x0670
        || is_between(code, 0x06D6, 0x06DC)
        || is_between(code, 0x06DF, 0x06E4)
        || is_between(code, 0x06E7, 0x06E8)
        || is_between(code, 0x06EA, 0x06ED)
        || code == 0x0711
        || is_between(code, 0x0730, 0x074A)
}
fn zero_width_marks_indic_a(code: u32) -> bool {
    is_between(code, 0x07A6, 0x07B0)
        || is_between(code, 0x07EB, 0x07F3)
        || is_between(code, 0x0816, 0x0819)
        || is_between(code, 0x081B, 0x0823)
        || is_between(code, 0x0825, 0x0827)
        || is_between(code, 0x0829, 0x082D)
        || is_between(code, 0x0859, 0x085B)
        || is_between(code, 0x08D3, 0x08E1)
        || is_between(code, 0x08E3, 0x0902)
        || is_between(code, 0x093A, 0x093C)
        || is_between(code, 0x0941, 0x0948)
        || code == 0x094D
        || is_between(code, 0x0951, 0x0957)
        || is_between(code, 0x0962, 0x0963)
        || is_between(code, 0x0981, 0x0983)
        || code == 0x09BC
        || is_between(code, 0x09BE, 0x09C4)
        || is_between(code, 0x09C7, 0x09C8)
        || is_between(code, 0x09CB, 0x09CD)
        || is_between(code, 0x09D7, 0x09D7)
        || is_between(code, 0x09E2, 0x09E3)
}
fn zero_width_marks_indic_b(code: u32) -> bool {
    is_between(code, 0x0A01, 0x0A03)
        || code == 0x0A3C
        || is_between(code, 0x0A3E, 0x0A42)
        || is_between(code, 0x0A47, 0x0A48)
        || is_between(code, 0x0A4B, 0x0A4D)
        || code == 0x0A51
        || is_between(code, 0x0A70, 0x0A71)
        || code == 0x0A75
        || is_between(code, 0x0A81, 0x0A83)
        || code == 0x0ABC
        || is_between(code, 0x0ABE, 0x0AC5)
        || is_between(code, 0x0AC7, 0x0AC9)
        || is_between(code, 0x0ACB, 0x0ACD)
        || is_between(code, 0x0AE2, 0x0AE3)
        || is_between(code, 0x0B01, 0x0B03)
        || code == 0x0B3C
        || is_between(code, 0x0B3E, 0x0B44)
        || is_between(code, 0x0B47, 0x0B48)
        || is_between(code, 0x0B4B, 0x0B4D)
        || code == 0x0B57
        || is_between(code, 0x0B62, 0x0B63)
        || code == 0x0B82
        || code == 0x0BBE
        || code == 0x0BC0
        || code == 0x0BCD
        || is_between(code, 0x0BD7, 0x0BD7)
        || is_between(code, 0x0C00, 0x0C04)
        || code == 0x0C3E
        || is_between(code, 0x0C40, 0x0C44)
        || is_between(code, 0x0C46, 0x0C48)
        || is_between(code, 0x0C4A, 0x0C4D)
        || is_between(code, 0x0C55, 0x0C56)
        || is_between(code, 0x0C62, 0x0C63)
        || is_between(code, 0x0C81, 0x0C83)
        || code == 0x0CBC
        || is_between(code, 0x0CBE, 0x0CC4)
        || is_between(code, 0x0CC6, 0x0CC8)
        || is_between(code, 0x0CCA, 0x0CCD)
        || is_between(code, 0x0CD5, 0x0CD6)
        || is_between(code, 0x0CE2, 0x0CE3)
        || is_between(code, 0x0D01, 0x0D03)
        || code == 0x0D3E
        || is_between(code, 0x0D40, 0x0D44)
        || is_between(code, 0x0D46, 0x0D48)
        || is_between(code, 0x0D4A, 0x0D4D)
        || code == 0x0D57
        || is_between(code, 0x0D62, 0x0D63)
}
fn zero_width_marks_indic_c(code: u32) -> bool {
    code == 0x0D82
        || code == 0x0D83
        || is_between(code, 0x0DCA, 0x0DCA)
        || is_between(code, 0x0DD2, 0x0DD4)
        || code == 0x0DD6
        || is_between(code, 0x0E31, 0x0E31)
        || is_between(code, 0x0E34, 0x0E3A)
        || is_between(code, 0x0E47, 0x0E4E)
        || code == 0x0EB1
        || is_between(code, 0x0EB4, 0x0EB9)
        || is_between(code, 0x0EBB, 0x0EBC)
        || is_between(code, 0x0EC8, 0x0ECD)
        || is_between(code, 0x0F18, 0x0F19)
        || code == 0x0F35
        || code == 0x0F37
        || code == 0x0F39
        || is_between(code, 0x0F71, 0x0F84)
        || is_between(code, 0x0F86, 0x0F87)
        || is_between(code, 0x0F8D, 0x0F97)
        || is_between(code, 0x0F99, 0x0FBC)
        || code == 0x0FC6
}
fn zero_width_marks_myanmar(code: u32) -> bool {
    is_between(code, 0x102D, 0x1030)
        || is_between(code, 0x1032, 0x1037)
        || is_between(code, 0x1039, 0x103A)
        || is_between(code, 0x103D, 0x103E)
        || is_between(code, 0x1058, 0x1059)
        || is_between(code, 0x105E, 0x1060)
        || is_between(code, 0x1071, 0x1074)
        || code == 0x1082
        || is_between(code, 0x1085, 0x1086)
        || code == 0x108D
        || code == 0x109D
        || is_between(code, 0x135D, 0x135F)
        || is_between(code, 0x1712, 0x1714)
        || is_between(code, 0x1732, 0x1734)
        || is_between(code, 0x1752, 0x1753)
        || is_between(code, 0x1772, 0x1773)
        || is_between(code, 0x17B4, 0x17B5)
        || is_between(code, 0x17B7, 0x17BD)
        || code == 0x17C6
        || is_between(code, 0x17C9, 0x17D3)
        || code == 0x17DD
        || is_between(code, 0x180B, 0x180D)
        || code == 0x1885
        || code == 0x1886
        || is_between(code, 0x18A9, 0x18A9)
        || is_between(code, 0x1920, 0x1922)
        || is_between(code, 0x1927, 0x1928)
        || code == 0x1932
        || is_between(code, 0x1939, 0x193B)
        || is_between(code, 0x1A17, 0x1A18)
        || code == 0x1A1B
        || code == 0x1A56
        || is_between(code, 0x1A58, 0x1A5E)
        || code == 0x1A60
        || code == 0x1A62
        || is_between(code, 0x1A65, 0x1A6C)
        || is_between(code, 0x1A73, 0x1A7C)
        || code == 0x1A7F
        || is_between(code, 0x1AB0, 0x1AC0)
        || is_between(code, 0x1B00, 0x1B03)
        || code == 0x1B34
        || is_between(code, 0x1B36, 0x1B3A)
        || code == 0x1B3C
        || code == 0x1B42
        || is_between(code, 0x1B6B, 0x1B73)
        || is_between(code, 0x1B80, 0x1B81)
}
fn zero_width_marks_balinese(code: u32) -> bool {
    is_between(code, 0x1BA2, 0x1BA5)
        || is_between(code, 0x1BA8, 0x1BA9)
        || is_between(code, 0x1BAB, 0x1BAD)
        || code == 0x1BE6
        || is_between(code, 0x1BE8, 0x1BE9)
        || code == 0x1BED
        || is_between(code, 0x1BEF, 0x1BF1)
        || is_between(code, 0x1C2C, 0x1C33)
        || is_between(code, 0x1C36, 0x1C37)
        || is_between(code, 0x1CD0, 0x1CD2)
        || is_between(code, 0x1CD4, 0x1CE0)
        || is_between(code, 0x1CE2, 0x1CE8)
        || code == 0x1CED
        || code == 0x1CF4
        || is_between(code, 0x1CF8, 0x1CF9)
        || is_between(code, 0x1DC0, 0x1DF9)
        || is_between(code, 0x1DFB, 0x1DFF)
        || code == 0x200B
        || code == 0x200C
        || code == 0x200D
        || code == 0x200E
        || code == 0x200F
        || is_between(code, 0x2028, 0x202E)
        || is_between(code, 0x2060, 0x2064)
        || is_between(code, 0x206A, 0x206F)
        || is_between(code, 0x20D0, 0x20F0)
}
fn zero_width_marks_cjk(code: u32) -> bool {
    is_between(code, 0x2CEF, 0x2CF1)
        || code == 0x2D7F
        || is_between(code, 0x2DE0, 0x2DFF)
        || is_between(code, 0x302A, 0x302D)
        || is_between(code, 0x3099, 0x309A)
        || is_between(code, 0xA66F, 0xA672)
        || is_between(code, 0xA674, 0xA67D)
        || is_between(code, 0xA69E, 0xA69F)
        || is_between(code, 0xA6F0, 0xA6F1)
}
fn zero_width_marks_aaa(code: u32) -> bool {
    code == 0xA802
        || code == 0xA806
        || code == 0xA80B
        || is_between(code, 0xA825, 0xA826)
        || is_between(code, 0xA8C4, 0xA8C5)
        || is_between(code, 0xA8E0, 0xA8F1)
        || code == 0xA926
        || is_between(code, 0xA927, 0xA92D)
        || is_between(code, 0xA947, 0xA951)
        || is_between(code, 0xA980, 0xA982)
        || code == 0xA9B3
        || is_between(code, 0xA9B6, 0xA9B9)
        || is_between(code, 0xA9BC, 0xA9BD)
        || code == 0xA9E5
        || is_between(code, 0xAA29, 0xAA2E)
        || is_between(code, 0xAA31, 0xAA32)
        || is_between(code, 0xAA35, 0xAA36)
        || code == 0xAA43
        || code == 0xAA4C
        || code == 0xAA7C
        || is_between(code, 0xAAB0, 0xAAB0)
        || code == 0xAAB2
        || code == 0xAAB3
        || code == 0xAAB4
        || is_between(code, 0xAAB7, 0xAAB8)
        || code == 0xAABE
        || code == 0xAABF
        || code == 0xAAC1
        || code == 0xAAEC
        || code == 0xAAED
        || code == 0xAAF6
        || is_between(code, 0xABE5, 0xABE5)
        || is_between(code, 0xABE8, 0xABE8)
        || code == 0xABED
        || code == 0xFB1E
        || is_between(code, 0xFE00, 0xFE0F)
        || is_between(code, 0xFE20, 0xFE2F)
        || code == 0xFEFF
}
fn zero_width_marks_smp_a(code: u32) -> bool {
    is_between(code, 0x101FD, 0x101FD)
        || is_between(code, 0x102E0, 0x102E0)
        || is_between(code, 0x10376, 0x1037A)
        || is_between(code, 0x10A01, 0x10A03)
        || is_between(code, 0x10A05, 0x10A06)
        || is_between(code, 0x10A0C, 0x10A0F)
        || is_between(code, 0x10A38, 0x10A3A)
        || code == 0x10A3F
        || is_between(code, 0x10AE5, 0x10AE6)
        || is_between(code, 0x11001, 0x11001)
        || is_between(code, 0x11038, 0x11046)
        || is_between(code, 0x1107F, 0x11081)
        || is_between(code, 0x110B3, 0x110B6)
        || is_between(code, 0x110B9, 0x110BA)
        || is_between(code, 0x11100, 0x11102)
        || is_between(code, 0x11127, 0x1112B)
        || is_between(code, 0x1112D, 0x11134)
        || is_between(code, 0x11173, 0x11173)
        || is_between(code, 0x11180, 0x11181)
        || is_between(code, 0x111B6, 0x111BE)
        || is_between(code, 0x111CA, 0x111CC)
        || is_between(code, 0x1122F, 0x11231)
        || is_between(code, 0x11234, 0x11234)
        || is_between(code, 0x11236, 0x11237)
        || is_between(code, 0x1123E, 0x1123E)
        || is_between(code, 0x112DF, 0x112DF)
        || is_between(code, 0x112E3, 0x112EA)
        || is_between(code, 0x11300, 0x11303)
        || is_between(code, 0x1133B, 0x1133C)
        || code == 0x1133E
        || is_between(code, 0x11340, 0x11340)
        || is_between(code, 0x11357, 0x11357)
        || is_between(code, 0x11366, 0x1136C)
        || is_between(code, 0x11370, 0x11374)
        || is_between(code, 0x11438, 0x1143F)
        || is_between(code, 0x11442, 0x11444)
        || is_between(code, 0x11446, 0x11446)
        || is_between(code, 0x1145E, 0x1145E)
}
fn zero_width_marks_smp_b(code: u32) -> bool {
    is_between(code, 0x114B3, 0x114B8)
        || is_between(code, 0x114BA, 0x114BA)
        || is_between(code, 0x114BF, 0x114C0)
        || is_between(code, 0x114C2, 0x114C3)
        || is_between(code, 0x115B2, 0x115B5)
        || is_between(code, 0x115BC, 0x115BD)
        || is_between(code, 0x115BF, 0x115C0)
        || is_between(code, 0x115DC, 0x115DD)
        || is_between(code, 0x11633, 0x1163A)
        || is_between(code, 0x1163D, 0x1163D)
        || is_between(code, 0x1163F, 0x11640)
        || is_between(code, 0x116AB, 0x116AB)
        || is_between(code, 0x116AD, 0x116AD)
        || is_between(code, 0x116B0, 0x116B5)
        || code == 0x116B7
        || is_between(code, 0x1171D, 0x1171F)
        || is_between(code, 0x11722, 0x11725)
        || is_between(code, 0x11727, 0x1172B)
        || is_between(code, 0x1182F, 0x11837)
        || is_between(code, 0x11839, 0x1183A)
        || is_between(code, 0x11A01, 0x11A0A)
        || is_between(code, 0x11A33, 0x11A38)
        || is_between(code, 0x11A3B, 0x11A3E)
        || code == 0x11A47
        || is_between(code, 0x11A51, 0x11A56)
        || is_between(code, 0x11A59, 0x11A5B)
        || code == 0x11A8A
        || is_between(code, 0x11A8C, 0x11A96)
        || code == 0x11A98
        || code == 0x11A99
        || is_between(code, 0x11C30, 0x11C36)
        || is_between(code, 0x11C38, 0x11C3D)
        || is_between(code, 0x11C3F, 0x11C3F)
        || is_between(code, 0x11C92, 0x11CA7)
        || is_between(code, 0x11CAA, 0x11CB0)
        || is_between(code, 0x11CB2, 0x11CB3)
        || is_between(code, 0x11CB5, 0x11CB6)
        || is_between(code, 0x11D31, 0x11D36)
        || is_between(code, 0x11D3A, 0x11D3A)
        || is_between(code, 0x11D3C, 0x11D3D)
        || is_between(code, 0x11D3F, 0x11D45)
        || code == 0x11D47
}
fn zero_width_marks_smp_c(code: u32) -> bool {
    is_between(code, 0x16AF0, 0x16AF4)
        || is_between(code, 0x16B30, 0x16B36)
        || is_between(code, 0x16F8F, 0x16F92)
        || is_between(code, 0x1BC9D, 0x1BC9E)
        || is_between(code, 0x1D165, 0x1D169)
        || is_between(code, 0x1D16D, 0x1D172)
        || is_between(code, 0x1D17B, 0x1D182)
        || is_between(code, 0x1D185, 0x1D18B)
        || is_between(code, 0x1D1AA, 0x1D1AD)
        || is_between(code, 0x1D242, 0x1D244)
        || is_between(code, 0x1DA00, 0x1DA36)
        || is_between(code, 0x1DA3B, 0x1DA6C)
        || is_between(code, 0x1DA75, 0x1DA75)
        || is_between(code, 0x1DA84, 0x1DA84)
        || is_between(code, 0x1DA9B, 0x1DA9F)
        || is_between(code, 0x1DAA1, 0x1DAAF)
}
fn zero_width_marks_smp_d(code: u32) -> bool {
    is_between(code, 0x1E000, 0x1E006)
        || is_between(code, 0x1E008, 0x1E018)
        || is_between(code, 0x1E01B, 0x1E021)
        || is_between(code, 0x1E023, 0x1E024)
        || is_between(code, 0x1E026, 0x1E02A)
        || is_between(code, 0x1E8D0, 0x1E8D6)
        || is_between(code, 0x1E944, 0x1E94A)
        || is_between(code, 0xE0100, 0xE01EF)
}
pub(crate) fn is_east_asian_wide(code: u32) -> bool {
    east_asian_wide_symbols(code)
        || east_asian_wide_cjk(code)
        || east_asian_wide_fullwidth(code)
        || east_asian_wide_tangut(code)
        || east_asian_wide_emoji(code)
        || east_asian_wide_cjk_ext(code)
}
fn east_asian_wide_symbols(code: u32) -> bool {
    is_between(code, 0x1100, 0x115F)
        || is_between(code, 0x231A, 0x231B)
        || is_between(code, 0x2329, 0x232A)
        || is_between(code, 0x23E9, 0x23EC)
        || code == 0x23F0
        || code == 0x23F3
        || is_between(code, 0x25FD, 0x25FE)
        || is_between(code, 0x2614, 0x2615)
        || is_between(code, 0x2648, 0x2653)
        || code == 0x267F
        || code == 0x2693
        || code == 0x26A1
        || is_between(code, 0x26AA, 0x26AB)
        || is_between(code, 0x26BD, 0x26BE)
        || is_between(code, 0x26C4, 0x26C5)
        || code == 0x26CE
        || code == 0x26D4
        || code == 0x26EA
        || is_between(code, 0x26F2, 0x26F3)
        || code == 0x26F5
        || code == 0x26FA
        || code == 0x26FD
        || code == 0x2705
        || is_between(code, 0x270A, 0x270B)
        || code == 0x2728
        || code == 0x274C
        || code == 0x274E
        || is_between(code, 0x2753, 0x2755)
        || code == 0x2757
        || is_between(code, 0x2795, 0x2797)
        || code == 0x27B0
        || code == 0x27BF
        || is_between(code, 0x2B1B, 0x2B1C)
        || code == 0x2B50
        || code == 0x2B55
}
fn east_asian_wide_cjk(code: u32) -> bool {
    is_between(code, 0x2E80, 0x2E99)
        || is_between(code, 0x2E9B, 0x2EF3)
        || is_between(code, 0x2F00, 0x2FD5)
        || is_between(code, 0x2FF0, 0x2FFB)
        || is_between(code, 0x3000, 0x303E)
        || is_between(code, 0x3041, 0x3096)
        || is_between(code, 0x3099, 0x30FF)
        || is_between(code, 0x3105, 0x312F)
        || is_between(code, 0x3131, 0x318E)
        || is_between(code, 0x3190, 0x31BA)
        || is_between(code, 0x31C0, 0x31E3)
        || is_between(code, 0x31F0, 0x321E)
        || is_between(code, 0x3220, 0x3247)
        || is_between(code, 0x3250, 0x4DBF)
        || is_between(code, 0x4E00, 0xA4C6)
}
fn east_asian_wide_fullwidth(code: u32) -> bool {
    is_between(code, 0xA960, 0xA97C)
        || is_between(code, 0xAC00, 0xD7A3)
        || is_between(code, 0xF900, 0xFAFF)
        || is_between(code, 0xFE10, 0xFE19)
        || is_between(code, 0xFE30, 0xFE52)
        || is_between(code, 0xFE54, 0xFE66)
        || is_between(code, 0xFE68, 0xFE6B)
        || is_between(code, 0xFF00, 0xFF60)
        || is_between(code, 0xFFE0, 0xFFE6)
}
fn east_asian_wide_tangut(code: u32) -> bool {
    is_between(code, 0x16FE0, 0x16FE4)
        || is_between(code, 0x17000, 0x187F7)
        || is_between(code, 0x18800, 0x18CD5)
        || is_between(code, 0x1B000, 0x1B2FB)
}
fn east_asian_wide_emoji(code: u32) -> bool {
    code == 0x1F004
        || code == 0x1F0CF
        || code == 0x1F18E
        || is_between(code, 0x1F191, 0x1F19A)
        || is_between(code, 0x1F200, 0x1F202)
        || is_between(code, 0x1F210, 0x1F23B)
        || is_between(code, 0x1F240, 0x1F248)
        || is_between(code, 0x1F250, 0x1F251)
        || is_between(code, 0x1F260, 0x1F265)
        || is_between(code, 0x1F300, 0x1F320)
        || is_between(code, 0x1F32D, 0x1F335)
        || is_between(code, 0x1F337, 0x1F37C)
        || is_between(code, 0x1F37E, 0x1F393)
        || is_between(code, 0x1F3A0, 0x1F3CA)
        || is_between(code, 0x1F3CF, 0x1F3D3)
        || is_between(code, 0x1F3E0, 0x1F3F0)
        || is_between(code, 0x1F3F4, 0x1F3F4)
        || is_between(code, 0x1F3F8, 0x1F43E)
        || is_between(code, 0x1F440, 0x1F440)
        || is_between(code, 0x1F442, 0x1F4FC)
        || is_between(code, 0x1F4FF, 0x1F53D)
        || is_between(code, 0x1F54B, 0x1F54E)
        || is_between(code, 0x1F550, 0x1F567)
        || is_between(code, 0x1F57A, 0x1F57A)
        || is_between(code, 0x1F595, 0x1F596)
        || is_between(code, 0x1F5A4, 0x1F5A4)
        || is_between(code, 0x1F5FB, 0x1F64F)
        || is_between(code, 0x1F680, 0x1F6C5)
        || is_between(code, 0x1F6CC, 0x1F6CC)
        || is_between(code, 0x1F6D0, 0x1F6D2)
        || is_between(code, 0x1F6D5, 0x1F6D7)
        || is_between(code, 0x1F6EB, 0x1F6EC)
        || is_between(code, 0x1F6F4, 0x1F6FC)
        || is_between(code, 0x1F7E0, 0x1F7EB)
        || is_between(code, 0x1F90C, 0x1F93A)
        || is_between(code, 0x1F93C, 0x1F945)
        || is_between(code, 0x1F947, 0x1F978)
        || is_between(code, 0x1F97A, 0x1F9CB)
        || is_between(code, 0x1F9CD, 0x1F9FF)
        || is_between(code, 0x1FA70, 0x1FA74)
        || is_between(code, 0x1FA78, 0x1FA7A)
        || is_between(code, 0x1FA80, 0x1FA86)
        || is_between(code, 0x1FA90, 0x1FAA8)
        || is_between(code, 0x1FAB0, 0x1FAB6)
        || is_between(code, 0x1FAC0, 0x1FAC2)
        || is_between(code, 0x1FAD0, 0x1FAD6)
}
fn east_asian_wide_cjk_ext(code: u32) -> bool {
    is_between(code, 0x20000, 0x2FFFD) || is_between(code, 0x30000, 0x3FFFD)
}
