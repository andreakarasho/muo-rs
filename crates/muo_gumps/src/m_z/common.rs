//! Small helpers every gump file here uses.

use muo_content::gumps::builder::GumpBuilder;
use muo_content::gumps::legacy::LegacyLayout;
use muo_content::gumps::{Gump, send_gump_to_mobile};
use muo_content::text::HtmlText;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;

/// `sender.Mobile`.
pub fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

/// `to.SendGump(gump)`.
pub fn send(w: &mut World, to: MobileId, gump: impl Gump + 'static) {
    send_gump_to_mobile(w, to, Box::new(gump), false);
}

/// `to.SendGump(gump, singleton: true)`.
pub fn send_singleton(w: &mut World, to: MobileId, gump: impl Gump + 'static) {
    send_gump_to_mobile(w, to, Box::new(gump), true);
}

/// `Mobile.Name` (empty for none).
pub fn name_of(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|mm| (mm.ty.name)(w, m))
        .unwrap_or_default()
}

/// `mobile.NetState != null`.
pub fn has_net_state(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|mm| mm.net_state.is_some())
}

/// Draws what `TextDefinition.AddHtmlText(Gump, ..)` decided on.
pub fn draw_legacy(l: &mut LegacyLayout, t: Option<HtmlText>, x: i32, y: i32, w: i32, h: i32) {
    match t {
        None => {}
        Some(HtmlText::Html { text, back, scroll }) => l.add_html(x, y, w, h, &text, back, scroll),
        Some(HtmlText::Localized {
            number,
            args: Some(args),
            color,
            back,
            scroll,
        }) => l.add_html_localized_args(
            x,
            y,
            w,
            h,
            number,
            &args,
            color.unwrap_or(0x7FFF),
            back,
            scroll,
        ),
        Some(HtmlText::Localized {
            number,
            args: None,
            color: Some(c),
            back,
            scroll,
        }) => l.add_html_localized_color(x, y, w, h, number, c, back, scroll),
        Some(HtmlText::Localized {
            number,
            args: None,
            color: None,
            back,
            scroll,
        }) => l.add_html_localized(x, y, w, h, number, back, scroll),
    }
}

/// Draws what `TextDefinition.AddHtmlText(ref builder, ..)` decided on.
pub fn draw_builder(b: &mut GumpBuilder, t: Option<HtmlText>, x: i32, y: i32, w: i32, h: i32) {
    match t {
        None => {}
        Some(HtmlText::Html { text, back, scroll }) => b.add_html(x, y, w, h, &text, back, scroll),
        Some(HtmlText::Localized {
            number,
            args: Some(args),
            color,
            back,
            scroll,
        }) => b.add_html_localized_args(
            x,
            y,
            w,
            h,
            number,
            &args,
            color.unwrap_or(0x7FFF),
            back,
            scroll,
        ),
        Some(HtmlText::Localized {
            number,
            args: None,
            color: Some(c),
            back,
            scroll,
        }) => b.add_html_localized_color(x, y, w, h, number, c, back, scroll),
        Some(HtmlText::Localized {
            number,
            args: None,
            color: None,
            back,
            scroll,
        }) => b.add_html_localized(x, y, w, h, number, back, scroll),
    }
}

/// `Html.Color(text, 0xRRGGBB)`.
pub fn color_rgb(text: &str, rgb: i32) -> String {
    muo_primitives::html::color_rgb(text, rgb)
}

/// `value.ToString("N0")`: groups of three digits.
pub fn n0(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if value < 0 { format!("-{out}") } else { out }
}
