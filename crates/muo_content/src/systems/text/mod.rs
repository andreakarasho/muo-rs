//! Text definition extensions (`TextDefinitionExtensions`).
//!
//! The four `AddHtmlText` overloads draw into a gump; the gump classes are the Gumps crate's. Each overload here
//! decides what to draw ([`HtmlText`]) and the gump code does the call, so the decision logic is tested here.

use muo_io::text::TextDefinition;
use muo_primitives::MessageType;
use muo_primitives::html;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part2::messages as mm;
use muo_server::property_list::PropertyList;

/// What `AddHtmlText` asks the gump to draw.
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlText {
    /// `AddHtmlLocalized(x, y, w, h, number, [args,] color, back, scroll)`; `color` is `None` for the overload
    /// without one.
    Localized {
        number: i32,
        args: Option<String>,
        color: Option<i32>,
        back: bool,
        scroll: bool,
    },
    /// `AddHtml(x, y, w, h, text, back, scroll)`.
    Html {
        text: String,
        back: bool,
        scroll: bool,
    },
}

/// `AddHtmlText(ref DynamicGumpBuilder | ref StaticGumpBuilder, ..)`: 15-bit `numberColor`, 24-bit `stringColor`
/// as `<BASEFONT COLOR=#RRGGBB>`. `-1` is "no color". Empty text draws nothing.
pub fn add_html_text_builder(
    def: Option<&TextDefinition>,
    back: bool,
    scroll: bool,
    number_color: i32,
    string_color: i32,
) -> Option<HtmlText> {
    let def = def.filter(|d| !d.is_empty())?;
    if def.number > 0 {
        // 5 bits per RGB component (15 bit RGB)
        let color = (number_color >= 0).then_some(number_color);
        return Some(HtmlText::Localized {
            number: def.number,
            args: None,
            color,
            back,
            scroll,
        });
    }
    let s = def.string.as_deref().unwrap_or("");
    let text = if string_color != -1 {
        format!("<BASEFONT COLOR=#{string_color:06X}>{s}</BASEFONT>")
    } else {
        s.to_string()
    };
    Some(HtmlText::Html { text, back, scroll })
}

/// `AddHtmlText(Gump g, ..)` without args: the legacy gump. A string is colored with `String.Color(stringColor)`.
pub fn add_html_text_gump(
    def: Option<&TextDefinition>,
    back: bool,
    scroll: bool,
    number_color: i32,
    string_color: i32,
) -> Option<HtmlText> {
    let def = def.filter(|d| !d.is_empty())?;
    if def.number <= 0 {
        let s = def.string.as_deref().unwrap_or("");
        // 8 bits per RGB component (24 bit RGB)
        let text = if string_color >= 0 {
            html::color_rgb(s, string_color)
        } else {
            s.to_string()
        };
        return Some(HtmlText::Html { text, back, scroll });
    }
    let color = (number_color >= 0).then_some(number_color);
    Some(HtmlText::Localized {
        number: def.number,
        args: None,
        color,
        back,
        scroll,
    })
}

/// `AddHtmlText(Gump g, .., string args, ..)`: a number takes the args (default color 0x7FFF); a string is
/// `string.Format(def.String, args)`.
pub fn add_html_text_gump_args(
    def: Option<&TextDefinition>,
    args: &str,
    back: bool,
    scroll: bool,
    number_color: i32,
    string_color: i32,
) -> Option<HtmlText> {
    let def = def.filter(|d| !d.is_empty())?;
    if def.number > 0 {
        // 5 bits per RGB component (15 bit RGB)
        let color = if number_color >= 0 {
            number_color
        } else {
            0x7FFF
        };
        return Some(HtmlText::Localized {
            number: def.number,
            args: Some(args.to_string()),
            color: Some(color),
            back,
            scroll,
        });
    }
    let formatted = format_one(def.string.as_deref().unwrap_or(""), args);
    // 8 bits per RGB component (24 bit RGB)
    let text = if string_color >= 0 {
        html::color_rgb(&formatted, string_color)
    } else {
        formatted
    };
    Some(HtmlText::Html { text, back, scroll })
}

/// `string.Format(format, arg0)`: `{0}` (with an optional `,width` or `:format` ignored) is replaced, `{{` and
/// `}}` are braces. shortcut: a `{1}` or a malformed brace stays as text instead of failing.
fn format_one(format: &str, arg: &str) -> String {
    let mut out = String::with_capacity(format.len() + arg.len());
    let mut it = format.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '{' if it.peek() == Some(&'{') => {
                it.next();
                out.push('{');
            }
            '}' if it.peek() == Some(&'}') => {
                it.next();
                out.push('}');
            }
            '{' => {
                let mut spec = String::new();
                let mut closed = false;
                for d in it.by_ref() {
                    if d == '}' {
                        closed = true;
                        break;
                    }
                    spec.push(d);
                }
                let index = spec.split([',', ':']).next().unwrap_or("");
                if closed && index == "0" {
                    out.push_str(arg);
                } else {
                    out.push('{');
                    out.push_str(&spec);
                    if closed {
                        out.push('}');
                    }
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// `AddTo(list)`: a number is `list.Add(number)`, else the string is `list.Add(string)`.
pub fn add_to(def: Option<&TextDefinition>, list: &mut PropertyList) {
    let Some(def) = def else { return };
    if def.number > 0 {
        list.add(def.number);
    } else if let Some(s) = &def.string {
        list.add_text(s);
    }
}

/// `AddTo(list, number)`: `number` is the cliloc that takes the definition as its argument.
pub fn add_to_number(def: Option<&TextDefinition>, list: &mut PropertyList, number: i32) {
    let Some(def) = def else { return };
    if def.number > 0 {
        list.add_localized_number(number, def.number);
    } else if let Some(s) = &def.string {
        list.add_str(number, s);
    }
}

/// `SendMessageTo(m)`.
pub fn send_message_to(w: &mut World, def: Option<&TextDefinition>, m: MobileId) {
    let Some(def) = def else { return };
    if def.number > 0 {
        mm::send_localized_message(w, m, def.number, "", 0x3B2);
    } else if let Some(s) = &def.string {
        mm::send_message(w, m, s);
    }
}

/// `SendMessageTo(m, hue)`.
pub fn send_message_to_hue(w: &mut World, def: Option<&TextDefinition>, m: MobileId, hue: i32) {
    let Some(def) = def else { return };
    if def.number > 0 {
        mm::send_localized_message(w, m, def.number, "", hue);
    } else if let Some(s) = &def.string {
        mm::send_message_hue(w, m, hue, s);
    }
}

/// `PublicOverheadMessage(this TextDefinition, Mobile m, messageType, hue)`.
pub fn public_overhead_message_mobile(
    w: &mut World,
    def: Option<&TextDefinition>,
    m: MobileId,
    message_type: MessageType,
    hue: i32,
) {
    let Some(def) = def else { return };
    if def.number > 0 {
        mm::public_overhead_message_number(w, m, message_type, hue, def.number, "", true);
    } else if let Some(s) = &def.string {
        mm::public_overhead_message(
            w,
            m,
            message_type,
            hue,
            false,
            s,
            true,
            muo_server::mobile::AccessLevel::Player,
        );
    }
}

/// `PublicOverheadMessage(this TextDefinition, Item item, messageType, hue)`.
pub fn public_overhead_message_item(
    w: &mut World,
    def: Option<&TextDefinition>,
    item: ItemId,
    message_type: muo_server::packets::message::MessageType,
    hue: i32,
) {
    use muo_server::item::messages as im;
    let Some(def) = def else { return };
    if def.number > 0 {
        im::public_overhead_message_number(w, item, message_type, hue, def.number, "");
    } else if let Some(s) = &def.string {
        im::public_overhead_message(w, item, message_type, hue, false, s);
    }
}

/// `IsNullOrEmpty(this TextDefinition)`.
pub fn is_null_or_empty(def: Option<&TextDefinition>) -> bool {
    def.is_none_or(TextDefinition::is_empty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(v: i32) -> TextDefinition {
        TextDefinition::of_number(v)
    }

    fn s(v: &str) -> TextDefinition {
        TextDefinition::of_text(v)
    }

    #[test]
    fn empty_draws_nothing() {
        assert_eq!(add_html_text_builder(None, false, false, -1, -1), None);
        assert_eq!(
            add_html_text_gump(Some(&TextDefinition::EMPTY), false, false, -1, -1),
            None
        );
        assert!(
            is_null_or_empty(None)
                && is_null_or_empty(Some(&n(0)))
                && !is_null_or_empty(Some(&n(1)))
        );
    }

    #[test]
    fn builder_overload() {
        assert_eq!(
            add_html_text_builder(Some(&n(1042)), true, false, 0x7FFF, 5),
            Some(HtmlText::Localized {
                number: 1042,
                args: None,
                color: Some(0x7FFF),
                back: true,
                scroll: false
            })
        );
        assert_eq!(
            add_html_text_builder(Some(&s("hi")), false, true, -1, 0xFF00),
            Some(HtmlText::Html {
                text: "<BASEFONT COLOR=#00FF00>hi</BASEFONT>".into(),
                back: false,
                scroll: true
            })
        );
        assert_eq!(
            add_html_text_builder(Some(&s("hi")), false, false, -1, -1),
            Some(HtmlText::Html {
                text: "hi".into(),
                back: false,
                scroll: false
            })
        );
    }

    #[test]
    fn gump_overloads() {
        assert_eq!(
            add_html_text_gump(Some(&s("hi")), false, false, -1, 0x00FF00),
            Some(HtmlText::Html {
                text: "<BASEFONT COLOR=#00FF00>hi</CENTER>".into(),
                back: false,
                scroll: false
            })
        );
        assert_eq!(
            add_html_text_gump_args(Some(&n(1070)), "x", false, false, -1, -1),
            Some(HtmlText::Localized {
                number: 1070,
                args: Some("x".into()),
                color: Some(0x7FFF),
                back: false,
                scroll: false
            })
        );
        assert_eq!(
            add_html_text_gump_args(Some(&s("Hello {0}!")), "World", false, false, -1, -1),
            Some(HtmlText::Html {
                text: "Hello World!".into(),
                back: false,
                scroll: false
            })
        );
    }

    #[test]
    fn string_format_one() {
        assert_eq!(format_one("{{a}} {0,5} {0:X}", "z"), "{a} z z");
    }
}
