//! **The oracle's tables**: what the same independent implementation says
//! about every version, where a full matrix apiece would be forty fixtures to
//! read one number out of.
//!
//! [`FULL`] is a symbol filled to its exact byte capacity at every version
//! 1–40: the capacity itself, the side that results, the mask the penalty rules
//! choose, and how many modules come out dark. The last is a weak signature on
//! its own and a strong one in company — it moves if a codeword moves, if a
//! block splits differently, if an alignment pattern lands a module off, or if
//! the mask choice changes — and the three full matrices beside it are what say
//! the symbol is *right* rather than merely stable.
//!
//! [`ALIGNMENT`] is the standard's own table of alignment-pattern centres,
//! every row of it, because the arithmetic that reproduces it has one exception
//! and a rule with an exception is a rule that has to be checked everywhere.

/// Per version: (version, payload bytes, side, mask, dark modules).
pub(super) const FULL: [(usize, usize, usize, u8, usize); 40] = [
    (1, 14, 21, 4, 232),
    (2, 26, 25, 0, 302),
    (3, 42, 29, 0, 432),
    (4, 62, 33, 6, 571),
    (5, 84, 37, 2, 718),
    (6, 106, 41, 2, 835),
    (7, 122, 45, 0, 1024),
    (8, 152, 49, 7, 1244),
    (9, 180, 53, 0, 1416),
    (10, 213, 57, 0, 1638),
    (11, 251, 61, 0, 1908),
    (12, 287, 65, 0, 2034),
    (13, 331, 69, 2, 2420),
    (14, 362, 73, 7, 2666),
    (15, 412, 77, 1, 3049),
    (16, 450, 81, 0, 3368),
    (17, 504, 85, 3, 3708),
    (18, 560, 89, 2, 4010),
    (19, 624, 93, 1, 4401),
    (20, 666, 97, 4, 4816),
    (21, 711, 101, 7, 5220),
    (22, 779, 105, 1, 5673),
    (23, 857, 109, 1, 6071),
    (24, 911, 113, 3, 6526),
    (25, 997, 117, 4, 7026),
    (26, 1059, 121, 3, 7442),
    (27, 1125, 125, 2, 7984),
    (28, 1190, 129, 2, 8411),
    (29, 1264, 133, 2, 9000),
    (30, 1370, 137, 0, 9528),
    (31, 1452, 141, 4, 10038),
    (32, 1538, 145, 5, 10603),
    (33, 1628, 149, 2, 11249),
    (34, 1722, 153, 2, 11874),
    (35, 1809, 157, 0, 12462),
    (36, 1911, 161, 0, 13120),
    (37, 1989, 165, 1, 14267),
    (38, 2099, 169, 2, 14448),
    (39, 2213, 173, 5, 15227),
    (40, 2331, 177, 0, 15692),
];

/// Per version 2–40: the alignment-pattern centre coordinates.
pub(super) const ALIGNMENT: [(usize, &[usize]); 39] = [
    (2, &[6, 18]),
    (3, &[6, 22]),
    (4, &[6, 26]),
    (5, &[6, 30]),
    (6, &[6, 34]),
    (7, &[6, 22, 38]),
    (8, &[6, 24, 42]),
    (9, &[6, 26, 46]),
    (10, &[6, 28, 50]),
    (11, &[6, 30, 54]),
    (12, &[6, 32, 58]),
    (13, &[6, 34, 62]),
    (14, &[6, 26, 46, 66]),
    (15, &[6, 26, 48, 70]),
    (16, &[6, 26, 50, 74]),
    (17, &[6, 30, 54, 78]),
    (18, &[6, 30, 56, 82]),
    (19, &[6, 30, 58, 86]),
    (20, &[6, 34, 62, 90]),
    (21, &[6, 28, 50, 72, 94]),
    (22, &[6, 26, 50, 74, 98]),
    (23, &[6, 30, 54, 78, 102]),
    (24, &[6, 28, 54, 80, 106]),
    (25, &[6, 32, 58, 84, 110]),
    (26, &[6, 30, 58, 86, 114]),
    (27, &[6, 34, 62, 90, 118]),
    (28, &[6, 26, 50, 74, 98, 122]),
    (29, &[6, 30, 54, 78, 102, 126]),
    (30, &[6, 26, 52, 78, 104, 130]),
    (31, &[6, 30, 56, 82, 108, 134]),
    (32, &[6, 34, 60, 86, 112, 138]),
    (33, &[6, 30, 58, 86, 114, 142]),
    (34, &[6, 34, 62, 90, 118, 146]),
    (35, &[6, 30, 54, 78, 102, 126, 150]),
    (36, &[6, 24, 50, 76, 102, 128, 154]),
    (37, &[6, 28, 54, 80, 106, 132, 158]),
    (38, &[6, 32, 58, 84, 110, 136, 162]),
    (39, &[6, 26, 54, 82, 110, 138, 166]),
    (40, &[6, 30, 58, 86, 114, 142, 170]),
];
