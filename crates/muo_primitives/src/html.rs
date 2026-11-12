//! Gump text markup builders and `EscapeHtml`.

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
#[repr(u8)]
pub enum TextAlignment {
    #[default]
    Left = 0,
    Center = 1,
    Right = 2,
}

pub fn center(input: &str) -> String {
    format!("<CENTER>{input}</CENTER>")
}

/// `<CENTER><BASEFONT COLOR=#RRGGBB>..</BASEFONT></CENTER>`
pub fn center_rgb(input: &str, color: i32) -> String {
    format!("<CENTER><BASEFONT COLOR=#{color:06X}>{input}</BASEFONT></CENTER>")
}

/// As [`center_rgb`] with a color name or `#RRGGBB` text.
pub fn center_color(input: &str, color: &str) -> String {
    format!("<CENTER><BASEFONT COLOR={color}>{input}</BASEFONT></CENTER>")
}

// The two `color*` helpers close with `</CENTER>`, which the client accepts; kept byte for byte.
pub fn color_rgb(input: &str, color: i32) -> String {
    format!("<BASEFONT COLOR=#{color:06X}>{input}</CENTER>")
}

pub fn color(input: &str, color: &str) -> String {
    format!("<BASEFONT COLOR={color}>{input}</CENTER>")
}

pub fn right(input: &str) -> String {
    format!("<RIGHT>{input}</RIGHT>")
}

pub fn right_rgb(input: &str, color: i32) -> String {
    format!("<RIGHT><BASEFONT COLOR=#{color:06X}>{input}</BASEFONT></RIGHT>")
}

pub fn right_color(input: &str, color: &str) -> String {
    format!("<RIGHT><BASEFONT COLOR={color}>{input}</BASEFONT></RIGHT>")
}

/// `EscapeHtml`: `& < > " '` to entities.
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// `Html.Build`: optional alignment wrapper around an optional `BASEFONT` (color, size when >= 0, style when > 0).
pub fn build(text: &str, color: &str, size: i32, font_style: u8, align: TextAlignment) -> String {
    let mut s = String::with_capacity(61 + text.len() + color.len());
    match align {
        TextAlignment::Right => s.push_str("<RIGHT>"),
        TextAlignment::Center => s.push_str("<CENTER>"),
        TextAlignment::Left => {}
    }
    if !color.is_empty() || size > -1 || font_style > 0 {
        s.push_str("<BASEFONT");
        if !color.is_empty() {
            s.push_str(" COLOR=");
            s.push_str(color);
        }
        if size > -1 {
            s.push_str(&format!(" SIZE={size}"));
        }
        if font_style > 0 {
            s.push_str(&format!(" STYLE={font_style}"));
        }
        s.push('>');
        s.push_str(text);
        s.push_str("</BASEFONT>");
    } else {
        s.push_str(text);
    }
    match align {
        TextAlignment::Right => s.push_str("</RIGHT>"),
        TextAlignment::Center => s.push_str("</CENTER>"),
        TextAlignment::Left => {}
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_escaping_needed() {
        for s in [
            "",
            "Hello World",
            "Plain text without special characters",
            "123456789",
            "!@#$%^*()_+-=[]{}|;:,.?",
        ] {
            assert_eq!(escape_html(s), s);
        }
    }

    #[test]
    fn single_and_mixed() {
        for (i, o) in [
            ("<", "&lt;"),
            (">", "&gt;"),
            ("&", "&amp;"),
            ("\"", "&quot;"),
            ("'", "&#39;"),
        ] {
            assert_eq!(escape_html(i), o);
        }
        for (i, o) in [
            (
                "<p>Hello & goodbye</p>",
                "&lt;p&gt;Hello &amp; goodbye&lt;/p&gt;",
            ),
            ("&lt;already&gt;", "&amp;lt;already&amp;gt;"),
            (
                "<tag attr=\"value\" data='test'>",
                "&lt;tag attr=&quot;value&quot; data=&#39;test&#39;&gt;",
            ),
            ("a<b>c&d\"e'f", "a&lt;b&gt;c&amp;d&quot;e&#39;f"),
            ("&nbsp;", "&amp;nbsp;"),
            (
                "<<>>&&\"\"''",
                "&lt;&lt;&gt;&gt;&amp;&amp;&quot;&quot;&#39;&#39;",
            ),
        ] {
            assert_eq!(escape_html(i), o);
        }
    }

    #[test]
    fn long_and_unicode() {
        let input = format!("Start{}End{}Final", "<".repeat(100), "&".repeat(100));
        let expected = format!("Start{}End{}Final", "&lt;".repeat(100), "&amp;".repeat(100));
        assert_eq!(escape_html(&input), expected);
        assert_eq!(
            escape_html("Hello 世界 <test> & 🎉"),
            "Hello 世界 &lt;test&gt; &amp; 🎉"
        );
    }

    #[test]
    fn markup() {
        assert_eq!(center("x"), "<CENTER>x</CENTER>");
        assert_eq!(
            center_rgb("x", 0xFF00),
            "<CENTER><BASEFONT COLOR=#00FF00>x</BASEFONT></CENTER>"
        );
        assert_eq!(
            right_color("x", "red"),
            "<RIGHT><BASEFONT COLOR=red>x</BASEFONT></RIGHT>"
        );
        assert_eq!(
            color_rgb("x", 0xABCDEF),
            "<BASEFONT COLOR=#ABCDEF>x</CENTER>"
        );
        assert_eq!(build("t", "", -1, 0, TextAlignment::Left), "t");
        assert_eq!(
            build("t", "red", 3, 2, TextAlignment::Center),
            "<CENTER><BASEFONT COLOR=red SIZE=3 STYLE=2>t</BASEFONT></CENTER>"
        );
        assert_eq!(
            build("t", "", 0, 0, TextAlignment::Right),
            "<RIGHT><BASEFONT SIZE=0>t</BASEFONT></RIGHT>"
        );
    }
}
