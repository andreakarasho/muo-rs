//! Small helpers every gump file here uses.

use muo_content::gumps::{Gump, send_gump_to_mobile};
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

/// `Mobile.ToString()`: `{Serial} "{Name}"`.
pub fn mobile_string(w: &World, m: MobileId) -> String {
    w.mobile(m).map_or_else(String::new, |mm| {
        format!(
            "{} \"{}\"",
            mm.serial,
            (mm.ty.name)(w, m).unwrap_or_default()
        )
    })
}

/// `Mobile.Name` (empty for none).
pub fn name_of(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|mm| (mm.ty.name)(w, m))
        .unwrap_or_default()
}

use muo_content::gumps::builder::GumpBuilder;
use muo_content::gumps::legacy::LegacyLayout;
use muo_content::text::HtmlText;

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

/// `Map.Name`.
pub fn map_name(w: &World, map: muo_server::map::MapId) -> String {
    w.maps
        .get(map)
        .map_or_else(String::new, |m| m.name().to_owned())
}
