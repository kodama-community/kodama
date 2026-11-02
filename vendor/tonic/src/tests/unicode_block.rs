// Ported from moonbit inspect/unicode_block_test.mbt
use crate::unicode::Block;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beside_preserves_rows_when_baselines_differ() {
        let top_aligned = Block::from("a\nb\nc");
        let bottom_aligned = Block {
            baseline: 2,
            ..Block::from("x\ny\nz")
        };

        let result = top_aligned.beside(&bottom_aligned);
        assert_eq!(result.render(), " x\n y\naz\nb \nc ");
        assert_eq!(result.height(), 5);
        assert_eq!(result.baseline(), 2);
        assert_eq!(result.width(), 2);

        let reversed = bottom_aligned.beside(&top_aligned);
        assert_eq!(reversed.render(), "x \ny \nza\n b\n c");
        assert_eq!(reversed.baseline(), 2);
    }

    #[test]
    fn t0() {
        let b = Block::from("hello");
        assert_eq!(b.render(), "hello");
        assert_eq!(b.height(), 1);
        assert_eq!(b.width(), 5);
    }

    #[test]
    fn t1() {
        let b = Block::empty();
        assert_eq!(b.render(), "");
        assert_eq!(b.height(), 0);
        assert_eq!(b.width(), 0);
    }

    #[test]
    fn t2() {
        let b = Block::from("ab");
        let padded = b.pad_to(5);
        assert_eq!(padded.render(), "ab   ");
        assert_eq!(padded.width(), 5);
        assert_eq!(b.pad_to(2).render(), "ab");
        assert_eq!(b.pad_to(0).render(), "ab");
    }

    #[test]
    fn t3() {
        let b = Block::from("ab");
        let centered = b.center(6);
        assert_eq!(centered.render(), "  ab  ");
        assert_eq!(centered.width(), 6);
        let b2 = Block::from("a");
        assert_eq!(b2.center(4).render(), " a  ");
    }

    #[test]
    fn t4() {
        let a = Block::from("ab");
        let b = Block::from("cd");
        let stacked = a.above(&b);
        assert_eq!(stacked.render(), "ab\ncd");
        assert_eq!(stacked.height(), 2);
        assert_eq!(stacked.width(), 2);
    }

    #[test]
    fn t5() {
        let a = Block::from("ab");
        let b = Block::from("xyz");
        let stacked = a.above(&b);
        assert_eq!(stacked.render(), "ab \nxyz");
        assert_eq!(stacked.width(), 3);
    }

    #[test]
    fn t6() {
        let a = Block::from("xyz");
        let b = Block::from("a");
        let stacked = a.above(&b);
        assert_eq!(stacked.render(), "xyz\na  ");
        assert_eq!(stacked.width(), 3);
    }

    #[test]
    fn t7() {
        let a = Block::from("ab");
        let b = Block::from("xy");
        let side = a.beside(&b);
        assert_eq!(side.render(), "abxy");
        assert_eq!(side.height(), 1);
        assert_eq!(side.width(), 4);
    }

    #[test]
    fn t8() {
        let b = Block::from("abc");
        let enclosed = b.enclose("(", ")");
        assert_eq!(enclosed.render(), "(abc)");
        assert_eq!(enclosed.width(), 5);
    }

    #[test]
    fn t9() {
        let b = Block::from("aa").above(&Block::from("b"));
        let enclosed = b.enclose("[", "]");
        assert_eq!(enclosed.render(), "[aa]\n[b ]");
    }

    #[test]
    fn t10() {
        let rows = Block::from("a  b").above(&Block::from("c  d"));
        let matrix = rows.enclose("[", "]");
        let label = Block::from("M = ");
        let result = matrix.append_left(&label);
        assert_eq!(result.render(), "M = [a  b]\n    [c  d]");
    }

    #[test]
    fn t11() {
        let rows = Block::from("a")
            .above(&Block::from("b"))
            .above(&Block::from("c"));
        let matrix = rows.enclose("[", "]");
        let label = Block::from("M = ");
        let result = matrix.append_left(&label);
        assert_eq!(result.render(), "M = [a]\n    [b]\n    [c]");
    }

    #[test]
    fn t12() {
        let rows = Block::from("aa  b  ").above(&Block::from("c   ddd"));
        let matrix = rows.enclose("[", "]");
        let label = Block::from("M = ");
        let result = matrix.append_left(&label);
        assert_eq!(result.render(), "M = [aa  b  ]\n    [c   ddd]");
    }

    #[test]
    fn t13() {
        let num = Block::from("p").center(3);
        let bar = Block::from("---");
        let den = Block::from("q").center(3);
        let frac = Block {
            baseline: 1,
            ..num.above(&bar).above(&den)
        };
        let label = Block::from("r = ");
        let result = frac.append_left(&label);
        assert_eq!(result.render(), "     p \nr = ---\n     q ");
    }

    #[test]
    fn t14() {
        let num = Block::from("p²").center(5);
        let bar = Block::from("-----");
        let den = Block::from("q+1").center(5);
        let frac = Block {
            baseline: 1,
            ..num.above(&bar).above(&den)
        };
        let label = Block::from("r = ");
        let result = frac.append_left(&label);
        assert_eq!(result.render(), "     p²  \nr = -----\n     q+1 ");
    }

    #[test]
    fn t15() {
        let left = Block::from("[a]").above(&Block::from("[b]"));
        let right = Block::from("[x]").above(&Block::from("[y]"));
        let result = left.beside(&right);
        assert_eq!(result.render(), "[a][x]\n[b][y]");
    }

    #[test]
    fn t16() {
        let left = Block::from("[a]").above(&Block::from("[b]"));
        let right = Block::from("[x]")
            .above(&Block::from("[y]"))
            .above(&Block::from("[z]"));
        let result = left.beside(&right);
        assert_eq!(result.render(), "[a][x]\n[b][y]\n   [z]");
    }

    #[test]
    fn t17() {
        let rows = Block::from("a  b").above(&Block::from("c  d"));
        let matrix = rows.enclose("[", "]");
        let tag = Block::from(",tag");
        let result = matrix.append_right(&tag);
        assert_eq!(result.render(), "[a  b],tag\n[c  d]    ");
    }

    #[test]
    fn t18() {
        let rows = Block::from("a  b").above(&Block::from("c  d"));
        let matrix = rows.enclose("(", ")");
        assert_eq!(matrix.render(), "(a  b)\n(c  d)");
    }

    #[test]
    fn t19() {
        let rows = Block::from("a").above(&Block::from("b"));
        let matrix = rows.enclose("(", ")");
        assert_eq!(matrix.render(), "(a)\n(b)");
    }
}
