use super::test_markdown_html;

#[test]
fn metadata_blocks_test_1() {
    let original = r##"---
title: example
another_field: 0
---
"##;
    let expected = r##""##;

    test_markdown_html(original, expected, false, true, false, false, false, false);
}

#[test]
#[should_panic(expected = "invalid emphasis at bytes 26..27: unclosed delimiter \"_\"")]
fn metadata_blocks_test_2() {
    let original = r##"---
title: example
another_field: 0
"##;
    test_markdown_html(original, "", false, true, false, false, false, false);
}

#[test]
fn metadata_blocks_test_3() {
    let original = r##"---
---
"##;
    let expected = r##"<hr>
<hr>
"##;

    test_markdown_html(original, expected, false, true, false, false, false, false);
}

#[test]
#[should_panic(expected = "invalid emphasis at bytes 27..28: unclosed delimiter \"_\"")]
fn metadata_blocks_test_4() {
    let original = r##"---

title: example
another_field: 0
---
"##;
    test_markdown_html(original, "", false, true, false, false, false, false);
}

#[test]
fn metadata_blocks_test_5() {
    let original = r##"My paragraph here.
---
title: example
another_field: 0
---
"##;
    let expected = r##"<p>My paragraph here.</p>
"##;

    test_markdown_html(original, expected, false, true, false, false, false, false);
}

#[test]
fn metadata_blocks_test_6() {
    let original = r##"My paragraph here.

---
title: example
another_field: 0
---
"##;
    let expected = r##"<p>My paragraph here.</p>
"##;

    test_markdown_html(original, expected, false, true, false, false, false, false);
}

#[test]
#[should_panic(expected = "invalid emphasis at bytes 73..74: unclosed delimiter \"_\"")]
fn metadata_blocks_test_7() {
    let original = r##"---    
title: example
another_field: 0
---

--- -
title: example
another_field: 0
---
"##;
    test_markdown_html(original, "", false, true, false, false, false, false);
}

#[test]
#[should_panic(expected = "invalid emphasis at bytes 75..76: unclosed delimiter \"_\"")]
fn metadata_blocks_test_8() {
    let original = r##"---
title: example
another_field: 0
---        

---
title: example
another_field: 0
---a
"##;
    test_markdown_html(original, "", false, true, false, false, false, false);
}

#[test]
fn metadata_blocks_test_9() {
    let original = r##"---
title: example
another_field: 0
...
"##;
    let expected = r##""##;

    test_markdown_html(original, expected, false, true, false, false, false, false);
}

#[test]
fn metadata_blocks_test_10() {
    let original = r##"+++
title: example
another_field: 0
+++
"##;
    let expected = r##""##;

    test_markdown_html(original, expected, false, true, false, false, false, false);
}

#[test]
fn metadata_blocks_test_11() {
    let original = r##"    ---
    Things
    ---
"##;
    let expected = r##"<p>---
Things
---</p>
"##;

    test_markdown_html(original, expected, false, true, false, false, false, false);
}

#[test]
fn metadata_blocks_test_12() {
    let original = r##"---
- Item 1
- Item 2
---
"##;
    let expected = r##""##;

    test_markdown_html(original, expected, false, true, false, false, false, false);
}
