//! `CustomHuePicker` and `MetallicHuePicker`: the restricted hue lists of
//! the leather, special, runebook and statuette tubs and the metallic tubs' picker. The callbacks' state is what the
//! closure captures.

use std::rc::Rc;

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::gumps::layout::GumpButtonType;
use muo_content::gumps::send_gump_to_mobile;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;

/// `CustomHuePickerCallback<T>(from, state, hue)`.
pub type HueCallback = Rc<dyn Fn(&mut World, MobileId, i32)>;

/// `CustomHueGroup` (a localized name).
pub struct HueGroup {
    pub name: i32,
    pub hues: &'static [i32],
}

/// `CustomHuePicker`.
pub struct CustomHuePicker {
    pub groups: &'static [HueGroup],
    pub default_supported: bool,
    pub title: i32,
}

const fn g(name: i32, hues: &'static [i32]) -> HueGroup {
    HueGroup { name, hues }
}

/// `CustomHuePicker.SpecialDyeTub`.
pub static SPECIAL_DYE_TUB: CustomHuePicker = CustomHuePicker {
    groups: &[
        g(1018345, &[1230, 1231, 1232, 1233, 1234, 1235]), // Violet
        g(1018346, &[1501, 1502, 1503, 1504, 1505, 1506, 1507, 1508]), // Tan
        g(1018347, &[2012, 2013, 2014, 2015, 2016, 2017]), // Brown
        g(1018348, &[1303, 1304, 1305, 1306, 1307, 1308]), // Dark Blue
        g(1018349, &[1420, 1421, 1422, 1423, 1424, 1425, 1426]), // Forest Green
        g(1018350, &[1619, 1620, 1621, 1622, 1623, 1624, 1625, 1626]), // Pink
        g(1018351, &[1640, 1641, 1642, 1643, 1644]),       // Red
        g(1018352, &[2001, 2002, 2003, 2004, 2005]),       // Olive
    ],
    default_supported: false,
    title: 1018344,
};

/// `CustomHuePicker.LeatherDyeTub`.
pub static LEATHER_DYE_TUB: CustomHuePicker = CustomHuePicker {
    groups: &[
        g(1018332, &[2419, 2420, 2421, 2422, 2423, 2424]), // Dull Copper
        g(1018333, &[2406, 2407, 2408, 2409, 2410, 2411, 2412]), // Shadow Iron
        g(1018334, &[2413, 2414, 2415, 2416, 2417, 2418]), // Copper
        g(1018335, &[2414, 2415, 2416, 2417, 2418]),       // Bronze
        g(1018336, &[2213, 2214, 2215, 2216, 2217, 2218]), // Golden
        g(1018337, &[2425, 2426, 2427, 2428, 2429, 2430]), // Agapite
        g(1018338, &[2207, 2208, 2209, 2210, 2211, 2212]), // Verite
        g(1018339, &[2219, 2220, 2221, 2222, 2223, 2224]), // Valorite
        g(1018340, &[2113, 2114, 2115, 2116, 2117, 2118]), // Reds
        g(1018341, &[2119, 2120, 2121, 2122, 2123, 2124]), // Blues
        g(1018342, &[2126, 2127, 2128, 2129, 2130]),       // Greens
        g(1018343, &[2213, 2214, 2215, 2216, 2217, 2218]), // Yellows
    ],
    default_supported: true,
    title: 0,
};

/// `CustomHuePickerGump<T>`.
pub struct CustomHuePickerGump {
    from: MobileId,
    def: &'static CustomHuePicker,
    callback: HueCallback,
}

/// `GetRadioID(group, index)`.
fn radio_id(def: &CustomHuePicker, group: usize, index: usize) -> i32 {
    (index * def.groups.len() + group) as i32
}

/// `new CustomHuePickerGump<T>(from, definition, callback, state)`.
pub fn new_custom(
    from: MobileId,
    def: &'static CustomHuePicker,
    callback: HueCallback,
) -> LegacyGumpObj<CustomHuePickerGump> {
    let mut g = LegacyGumpObj::new(
        50,
        50,
        CustomHuePickerGump {
            from,
            def,
            callback,
        },
    );
    let l = &mut g.layout;
    // RenderBackground
    l.add_page(0);
    l.add_background(0, 0, 450, 450, 5054);
    l.add_background(10, 10, 430, 430, 3000);
    if def.title > 0 {
        l.add_html_localized(20, 30, 400, 25, def.title, false, false);
    }
    l.add_button(20, 400, 4005, 4007, 1);
    l.add_html_localized(55, 400, 200, 25, 1011036, false, false); // OKAY
    if def.default_supported {
        l.add_button(200, 400, 4005, 4007, 2);
        l.add_label(235, 400, 0, "DEFAULT");
    }
    // RenderCategories
    for (i, grp) in def.groups.iter().enumerate() {
        let y = 85 + i as i32 * 25;
        l.add_button_type(30, y, 5224, 5224, 0, GumpButtonType::Page, 1 + i as i32);
        l.add_html_localized(55, y, 200, 25, grp.name, false, false);
    }
    for (i, grp) in def.groups.iter().enumerate() {
        l.add_page(1 + i as i32);
        for (j, hue) in grp.hues.iter().enumerate() {
            let y = 90 + j as i32 * 25;
            l.add_radio(260, y, 210, 211, false, radio_id(def, i, j));
            l.add_label(278, y, hue - 1, "*****");
        }
    }
    g
}

impl LegacyGumpLogic for CustomHuePickerGump {
    const TYPE_NAME: &'static str = "Server.Items.CustomHuePickerGump`1";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        match info.button_id {
            1 => {
                // Okay
                if let Some(&index) = info.switches.first() {
                    let n = self.def.groups.len() as i32;
                    let group = index % n;
                    let index = index / n;
                    if group >= 0 && group < n {
                        let hues = self.def.groups[group as usize].hues;
                        if index >= 0 && (index as usize) < hues.len() {
                            (self.callback.clone())(w, self.from, hues[index as usize]);
                        }
                    }
                }
            }
            2
                // Default
                if self.def.default_supported => {
                    (self.callback.clone())(w, self.from, 0);
                }
            _ => {}
        }
    }
}

/// `MetallicHuePicker<T>`.
pub struct MetallicHuePicker {
    from: MobileId,
    callback: HueCallback,
}

/// `new MetallicHuePicker<T>(from, callback, state)`.
pub fn new_metallic(from: MobileId, callback: HueCallback) -> LegacyGumpObj<MetallicHuePicker> {
    let mut g = LegacyGumpObj::new(450, 450, MetallicHuePicker { from, callback });
    let l = &mut g.layout;
    l.add_page(0);
    l.add_background(0, 0, 450, 450, 0x13BE);
    l.add_background(10, 10, 430, 430, 0xBB8);
    l.add_html_localized(55, 400, 200, 25, 1011036, false, false); // OKAY
    l.add_button(20, 400, 4005, 4007, 1);
    l.add_button(200, 400, 4005, 4007, 2);
    l.add_label(235, 400, 0, "DEFAULT");
    l.add_html_localized(55, 25, 200, 25, 1150063, false, false); // Base/Shadow Color
    l.add_html_localized(260, 25, 200, 25, 1150064, false, false); // Highlight Color
    for row in 0..13 {
        l.add_button_type(
            30,
            65 + row * 25,
            0x1467,
            0x1468,
            row + 1,
            GumpButtonType::Page,
            row + 1,
        );
        l.add_item(
            50,
            65 + row * 25,
            0x1412,
            2501 + row * 12 + if row == 12 { 6 } else { 0 },
        );
    }
    for page in 1..14 {
        l.add_page(page);
        for row in 0..12 {
            // OSI just had to skip 6 unused hues, didnt they
            let hue = 2501 + if page == 13 { 6 } else { 0 } + row + 12 * (page - 1);
            l.add_radio(260, 65 + row * 25, 0xd2, 0xd3, false, hue);
            l.add_item(280, 65 + row * 25, 0x1412, hue);
        }
    }
    g
}

impl LegacyGumpLogic for MetallicHuePicker {
    const TYPE_NAME: &'static str = "Server.Items.MetallicHuePicker`1";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        match info.button_id {
            1 => {
                if let Some(&h) = info.switches.first() {
                    (self.callback.clone())(w, self.from, h);
                }
            }
            2 => (self.callback.clone())(w, self.from, 0),
            _ => {}
        }
    }
}

/// `to.SendGump(gump)`.
pub fn send(w: &mut World, to: MobileId, gump: impl muo_content::gumps::Gump + 'static) {
    send_gump_to_mobile(w, to, Box::new(gump), false);
}
