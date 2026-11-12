//! `AddDoorGump`.

use std::rc::Rc;

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, send};

const TYPES: [&[(&str, i32)]; 4] = [
    // Standard
    &[
        ("MetalDoor", 0x675),
        ("RattanDoor", 0x695),
        ("DarkWoodDoor", 0x6A5),
        ("LightWoodDoor", 0x6D5),
        ("StrongWoodDoor", 0x6E5),
        ("BarredMetalDoor2", 0x1FED),
    ],
    // Tall
    &[
        ("BarredMetalDoor", 0x685),
        ("MediumWoodDoor", 0x6B5),
        ("MetalDoor2", 0x6C5),
    ],
    // Short
    &[
        ("IronGate", 0x824),
        ("IronGateShort", 0x84C),
        ("LightWoodGate", 0x839),
        ("DarkWoodGate", 0x866),
    ],
    // Secret
    &[
        ("SecretStoneDoor1", 0xE8),
        ("SecretDungeonDoor", 0x314),
        ("SecretStoneDoor2", 0x324),
        ("SecretWoodenDoor", 0x334),
        ("SecretLightWoodDoor", 0x344),
        ("SecretStoneDoor3", 0x354),
    ],
];

/// `DoorFacing`'s names, by value.
const FACINGS: [&str; 8] = [
    "WestCW", "EastCCW", "WestCCW", "EastCW", "SouthCW", "OutCCW", "SouthCCW", "NorthCW",
];

#[derive(Clone)]
pub struct AddDoorGump {
    ty: Option<&'static str>,
    base_id: i32,
    page: usize,
}

/// `new AddDoorGump()`.
pub fn new() -> DynamicGumpObj<AddDoorGump> {
    DynamicGumpObj::new(
        50,
        40,
        AddDoorGump {
            ty: None,
            base_id: 0,
            page: 0,
        },
    )
}

/// `AddDoorGump.Configure`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "AddDoor",
        AccessLevel::GameMaster,
        &[],
        Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            // Usage: AddDoor; Description: Displays a menu from which you can interactively add doors.
            send(w, e.mobile, new());
        }),
    );
}

fn add_blue_back(b: &mut DynamicGumpBuilder, width: i32, height: i32) {
    b.add_background(0, 0, width, height, 0xE10);
    b.add_background(8, 5, width - 16, height - 11, 0x053);
    b.add_image_tiled(15, 14, width - 29, height - 29, 0xE14);
    b.add_alpha_region(15, 14, width - 29, height - 29);
}

impl DynamicGumpLogic for AddDoorGump {
    const TYPE_NAME: &'static str = "Server.Gumps.AddDoorGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);

        if self.ty.is_some() {
            add_blue_back(b, 155, 174);
            let id = self.base_id;
            b.add_item(25, 24, id, 0);
            b.add_button(26, 37, 0x5782, 0x5782, 10);
            b.add_item(47, 45, id + 2, 0);
            b.add_button(43, 57, 0x5783, 0x5783, 11);
            b.add_item(87, 22, id + 10, 0);
            b.add_button(116, 35, 0x5785, 0x5785, 15);
            b.add_item(65, 45, id + 8, 0);
            b.add_button(96, 55, 0x5784, 0x5784, 14);
            b.add_button(73, 36, 0x2716, 0x2716, 18);
        } else {
            let types = TYPES[self.page];
            add_blue_back(b, 20 + (types.len() as i32 + 1) * 50, 165);
            b.add_html_localized_color(30, 45, 60, 20, 1043353, 0x7FFF, false, false); // Next
            b.add_html_localized_color(30, 85, 60, 20, 1011393, 0x7FFF, false, false); // Back
            b.add_button(30, 60, 0xFA5, 0xFA7, 1);
            b.add_button(30, 100, 0xFAE, 0xFB0, 2);
            for (i, (_, base)) in types.iter().enumerate() {
                let x = (i as i32 + 1) * 50;
                b.add_button(
                    30 + x,
                    20,
                    0x2624,
                    0x2625,
                    (self.page * types.len() + i) as i32 + 10,
                );
                b.add_item(15 + x, 30, *base, 0);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let mut button = info.button_id;

        if button == 0 {
            if self.ty.is_some() {
                self.ty = None;
                self.base_id = 0;
            } else {
                return;
            }
        } else if button == 1 {
            self.page += 1;
            if self.page >= TYPES.len() {
                self.page = 0;
            }
        } else if button == 2 {
            self.page = if self.page == 0 {
                TYPES.len() - 1
            } else {
                self.page - 1
            };
        } else {
            button -= 10;
            match self.ty {
                None => {
                    let types = TYPES[self.page];
                    let (page, pos) = (button / types.len() as i32, button % types.len() as i32);
                    if page < 0 || pos < 0 || pos >= types.len() as i32 {
                        return;
                    }
                    let door = types[pos as usize];
                    self.ty = Some(door.0);
                    self.base_id = door.1;
                }
                Some(ty) if (0..8).contains(&button) => {
                    let prefix = w.resources.get_mut::<CommandSystem>().prefix.clone();
                    let text = format!("{prefix}Add {ty} {}", FACINGS[button as usize]);
                    CommandSystem::handle(w, from, &text, MessageType::REGULAR);
                }
                Some(_) if button == 8 => {
                    let prefix = w.resources.get_mut::<CommandSystem>().prefix.clone();
                    CommandSystem::handle(w, from, &format!("{prefix}Link"), MessageType::REGULAR);
                }
                Some(_) => {}
            }
        }

        // from.SendGump(this): the gump was removed from the open list for this response; send the same state again
        send(w, from, DynamicGumpObj::new(50, 40, self.clone()));
    }
}
