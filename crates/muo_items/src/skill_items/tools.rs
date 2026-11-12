//! The plain craft and harvest tools. A tool's state is `muo_content::items::tool::ToolData`; the class table says which system it opens.

use muo_content::items::tool::{
    self, BASE_HARVEST_TOOL, BASE_HARVEST_TOOL_CLASS, BASE_TOOL, BASE_TOOL_CLASS, ToolClass,
};
use muo_server::item::ItemType;
use muo_server::layer::Layer;
use muo_server::type_registry::TypeRegistry;

use super::{read_version, write_version};

/// `Server.Items.Blowpipe`.
pub static BLOWPIPE_CLASS: ToolClass = ToolClass {
    craft_system: "Glassblowing",
    ..BASE_TOOL_CLASS
};
pub static BLOWPIPE: ItemType = ItemType {
    full_name: "Server.Items.Blowpipe",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_blowpipe(w, None)),
    family: Some(&BLOWPIPE_CLASS),
    default_weight: |_, _| 4.0,
    label_number: |_, _| 1044608,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Blowpipe()` / `new Blowpipe(uses)`.
pub fn new_blowpipe(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    let id = match uses {
        Some(u) => tool::new_tool(w, &BLOWPIPE, 0xE8A, u),
        None => tool::new_tool_random(w, &BLOWPIPE, 0xE8A),
    };
    let ty = &BLOWPIPE;
    (ty.set_hue)(w, id, 0x3B9);
    id
}

/// `Server.Items.DovetailSaw`.
pub static DOVETAIL_SAW_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static DOVETAIL_SAW: ItemType = ItemType {
    full_name: "Server.Items.DovetailSaw",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_dovetail_saw(w, None)),
    family: Some(&DOVETAIL_SAW_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new DovetailSaw()` / `new DovetailSaw(uses)`.
pub fn new_dovetail_saw(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &DOVETAIL_SAW, 0x1028, u),
        None => tool::new_tool_random(w, &DOVETAIL_SAW, 0x1028),
    }
}

/// `Server.Items.DrawKnife`.
pub static DRAW_KNIFE_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static DRAW_KNIFE: ItemType = ItemType {
    full_name: "Server.Items.DrawKnife",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_draw_knife(w, None)),
    family: Some(&DRAW_KNIFE_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new DrawKnife()` / `new DrawKnife(uses)`.
pub fn new_draw_knife(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &DRAW_KNIFE, 0x10E4, u),
        None => tool::new_tool_random(w, &DRAW_KNIFE, 0x10E4),
    }
}

/// `Server.Items.FletcherTools`.
pub static FLETCHER_TOOLS_CLASS: ToolClass = ToolClass {
    craft_system: "BowFletching",
    ..BASE_TOOL_CLASS
};
pub static FLETCHER_TOOLS: ItemType = ItemType {
    full_name: "Server.Items.FletcherTools",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_fletcher_tools(w, None)),
    family: Some(&FLETCHER_TOOLS_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new FletcherTools()` / `new FletcherTools(uses)`.
pub fn new_fletcher_tools(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &FLETCHER_TOOLS, 0x1022, u),
        None => tool::new_tool_random(w, &FLETCHER_TOOLS, 0x1022),
    }
}

/// `Server.Items.FlourSifter`.
pub static FLOUR_SIFTER_CLASS: ToolClass = ToolClass {
    craft_system: "Cooking",
    ..BASE_TOOL_CLASS
};
pub static FLOUR_SIFTER: ItemType = ItemType {
    full_name: "Server.Items.FlourSifter",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_flour_sifter(w, None)),
    family: Some(&FLOUR_SIFTER_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new FlourSifter()` / `new FlourSifter(uses)`.
pub fn new_flour_sifter(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &FLOUR_SIFTER, 0x103E, u),
        None => tool::new_tool_random(w, &FLOUR_SIFTER, 0x103E),
    }
}

/// `Server.Items.Froe`.
pub static FROE_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static FROE: ItemType = ItemType {
    full_name: "Server.Items.Froe",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_froe(w, None)),
    family: Some(&FROE_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Froe()` / `new Froe(uses)`.
pub fn new_froe(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &FROE, 0x10E5, u),
        None => tool::new_tool_random(w, &FROE, 0x10E5),
    }
}

/// `Server.Items.Hammer`.
pub static HAMMER_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static HAMMER: ItemType = ItemType {
    full_name: "Server.Items.Hammer",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_hammer(w, None)),
    family: Some(&HAMMER_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Hammer()` / `new Hammer(uses)`.
pub fn new_hammer(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &HAMMER, 0x102A, u),
        None => tool::new_tool_random(w, &HAMMER, 0x102A),
    }
}

/// `Server.Items.Inshave`.
pub static INSHAVE_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static INSHAVE: ItemType = ItemType {
    full_name: "Server.Items.Inshave",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_inshave(w, None)),
    family: Some(&INSHAVE_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Inshave()` / `new Inshave(uses)`.
pub fn new_inshave(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &INSHAVE, 0x10E6, u),
        None => tool::new_tool_random(w, &INSHAVE, 0x10E6),
    }
}

/// `Server.Items.JointingPlane`.
pub static JOINTING_PLANE_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static JOINTING_PLANE: ItemType = ItemType {
    full_name: "Server.Items.JointingPlane",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_jointing_plane(w, None)),
    family: Some(&JOINTING_PLANE_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new JointingPlane()` / `new JointingPlane(uses)`.
pub fn new_jointing_plane(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &JOINTING_PLANE, 0x1030, u),
        None => tool::new_tool_random(w, &JOINTING_PLANE, 0x1030),
    }
}

/// `Server.Items.MalletAndChisel`.
pub static MALLET_AND_CHISEL_CLASS: ToolClass = ToolClass {
    craft_system: "Masonry",
    ..BASE_TOOL_CLASS
};
pub static MALLET_AND_CHISEL: ItemType = ItemType {
    full_name: "Server.Items.MalletAndChisel",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_mallet_and_chisel(w, None)),
    family: Some(&MALLET_AND_CHISEL_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new MalletAndChisel()` / `new MalletAndChisel(uses)`.
pub fn new_mallet_and_chisel(
    w: &mut muo_server::World,
    uses: Option<i32>,
) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &MALLET_AND_CHISEL, 0x12B3, u),
        None => tool::new_tool_random(w, &MALLET_AND_CHISEL, 0x12B3),
    }
}

/// `Server.Items.MapmakersPen`.
pub static MAPMAKERS_PEN_CLASS: ToolClass = ToolClass {
    craft_system: "Cartography",
    ..BASE_TOOL_CLASS
};
pub static MAPMAKERS_PEN: ItemType = ItemType {
    full_name: "Server.Items.MapmakersPen",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_mapmakers_pen(w, None)),
    family: Some(&MAPMAKERS_PEN_CLASS),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1044167,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new MapmakersPen()` / `new MapmakersPen(uses)`.
pub fn new_mapmakers_pen(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &MAPMAKERS_PEN, 0x0FBF, u),
        None => tool::new_tool_random(w, &MAPMAKERS_PEN, 0x0FBF),
    }
}

/// `Server.Items.MortarPestle`.
pub static MORTAR_PESTLE_CLASS: ToolClass = ToolClass {
    craft_system: "Alchemy",
    ..BASE_TOOL_CLASS
};
pub static MORTAR_PESTLE: ItemType = ItemType {
    full_name: "Server.Items.MortarPestle",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_mortar_pestle(w, None)),
    family: Some(&MORTAR_PESTLE_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new MortarPestle()` / `new MortarPestle(uses)`.
pub fn new_mortar_pestle(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &MORTAR_PESTLE, 0xE9B, u),
        None => tool::new_tool_random(w, &MORTAR_PESTLE, 0xE9B),
    }
}

/// `Server.Items.MouldingPlane`.
pub static MOULDING_PLANE_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static MOULDING_PLANE: ItemType = ItemType {
    full_name: "Server.Items.MouldingPlane",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_moulding_plane(w, None)),
    family: Some(&MOULDING_PLANE_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new MouldingPlane()` / `new MouldingPlane(uses)`.
pub fn new_moulding_plane(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &MOULDING_PLANE, 0x102C, u),
        None => tool::new_tool_random(w, &MOULDING_PLANE, 0x102C),
    }
}

/// `Server.Items.Nails`.
pub static NAILS_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static NAILS: ItemType = ItemType {
    full_name: "Server.Items.Nails",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_nails(w, None)),
    family: Some(&NAILS_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Nails()` / `new Nails(uses)`.
pub fn new_nails(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &NAILS, 0x102E, u),
        None => tool::new_tool_random(w, &NAILS, 0x102E),
    }
}

/// `Server.Items.RollingPin`.
pub static ROLLING_PIN_CLASS: ToolClass = ToolClass {
    craft_system: "Cooking",
    ..BASE_TOOL_CLASS
};
pub static ROLLING_PIN: ItemType = ItemType {
    full_name: "Server.Items.RollingPin",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_rolling_pin(w, None)),
    family: Some(&ROLLING_PIN_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new RollingPin()` / `new RollingPin(uses)`.
pub fn new_rolling_pin(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &ROLLING_PIN, 0x1043, u),
        None => tool::new_tool_random(w, &ROLLING_PIN, 0x1043),
    }
}

/// `Server.Items.Saw`.
pub static SAW_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static SAW: ItemType = ItemType {
    full_name: "Server.Items.Saw",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_saw(w, None)),
    family: Some(&SAW_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Saw()` / `new Saw(uses)`.
pub fn new_saw(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &SAW, 0x1034, u),
        None => tool::new_tool_random(w, &SAW, 0x1034),
    }
}

/// `Server.Items.Scorp`.
pub static SCORP_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static SCORP: ItemType = ItemType {
    full_name: "Server.Items.Scorp",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_scorp(w, None)),
    family: Some(&SCORP_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Scorp()` / `new Scorp(uses)`.
pub fn new_scorp(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &SCORP, 0x10E7, u),
        None => tool::new_tool_random(w, &SCORP, 0x10E7),
    }
}

/// `Server.Items.ScribesPen`.
pub static SCRIBES_PEN_CLASS: ToolClass = ToolClass {
    craft_system: "Inscription",
    ..BASE_TOOL_CLASS
};
pub static SCRIBES_PEN: ItemType = ItemType {
    full_name: "Server.Items.ScribesPen",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_scribes_pen(w, None)),
    family: Some(&SCRIBES_PEN_CLASS),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1044168,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new ScribesPen()` / `new ScribesPen(uses)`.
pub fn new_scribes_pen(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &SCRIBES_PEN, 0x0FBF, u),
        None => tool::new_tool_random(w, &SCRIBES_PEN, 0x0FBF),
    }
}

/// `Server.Items.SewingKit`.
pub static SEWING_KIT_CLASS: ToolClass = ToolClass {
    craft_system: "Tailoring",
    ..BASE_TOOL_CLASS
};
pub static SEWING_KIT: ItemType = ItemType {
    full_name: "Server.Items.SewingKit",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_sewing_kit(w, None)),
    family: Some(&SEWING_KIT_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new SewingKit()` / `new SewingKit(uses)`.
pub fn new_sewing_kit(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &SEWING_KIT, 0xF9D, u),
        None => tool::new_tool_random(w, &SEWING_KIT, 0xF9D),
    }
}

/// `Server.Items.Skillet`.
pub static SKILLET_CLASS: ToolClass = ToolClass {
    craft_system: "Cooking",
    ..BASE_TOOL_CLASS
};
pub static SKILLET: ItemType = ItemType {
    full_name: "Server.Items.Skillet",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_skillet(w, None)),
    family: Some(&SKILLET_CLASS),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1044567,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Skillet()` / `new Skillet(uses)`.
pub fn new_skillet(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &SKILLET, 0x97F, u),
        None => tool::new_tool_random(w, &SKILLET, 0x97F),
    }
}

/// `Server.Items.SledgeHammer`.
pub static SLEDGE_HAMMER_CLASS: ToolClass = ToolClass {
    craft_system: "Blacksmithy",
    ..BASE_TOOL_CLASS
};
pub static SLEDGE_HAMMER: ItemType = ItemType {
    full_name: "Server.Items.SledgeHammer",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_sledge_hammer(w, None)),
    family: Some(&SLEDGE_HAMMER_CLASS),
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new SledgeHammer()` / `new SledgeHammer(uses)`.
pub fn new_sledge_hammer(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    let id = match uses {
        Some(u) => tool::new_tool(w, &SLEDGE_HAMMER, 0xFB5, u),
        None => tool::new_tool_random(w, &SLEDGE_HAMMER, 0xFB5),
    };
    let ty = &SLEDGE_HAMMER;
    (ty.set_layer)(w, id, Layer::OneHanded);
    id
}

/// `Server.Items.SmithHammer`.
pub static SMITH_HAMMER_CLASS: ToolClass = ToolClass {
    craft_system: "Blacksmithy",
    ..BASE_TOOL_CLASS
};
pub static SMITH_HAMMER: ItemType = ItemType {
    full_name: "Server.Items.SmithHammer",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_smith_hammer(w, None)),
    family: Some(&SMITH_HAMMER_CLASS),
    default_weight: |_, _| 8.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new SmithHammer()` / `new SmithHammer(uses)`.
pub fn new_smith_hammer(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    let id = match uses {
        Some(u) => tool::new_tool(w, &SMITH_HAMMER, 0x13E3, u),
        None => tool::new_tool_random(w, &SMITH_HAMMER, 0x13E3),
    };
    let ty = &SMITH_HAMMER;
    (ty.set_layer)(w, id, Layer::OneHanded);
    id
}

/// `Server.Items.SmoothingPlane`.
pub static SMOOTHING_PLANE_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};
pub static SMOOTHING_PLANE: ItemType = ItemType {
    full_name: "Server.Items.SmoothingPlane",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_smoothing_plane(w, None)),
    family: Some(&SMOOTHING_PLANE_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new SmoothingPlane()` / `new SmoothingPlane(uses)`.
pub fn new_smoothing_plane(
    w: &mut muo_server::World,
    uses: Option<i32>,
) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &SMOOTHING_PLANE, 0x1032, u),
        None => tool::new_tool_random(w, &SMOOTHING_PLANE, 0x1032),
    }
}

/// `Server.Items.TinkerTools`.
pub static TINKER_TOOLS_CLASS: ToolClass = ToolClass {
    craft_system: "Tinkering",
    ..BASE_TOOL_CLASS
};
pub static TINKER_TOOLS: ItemType = ItemType {
    full_name: "Server.Items.TinkerTools",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_tinker_tools(w, None)),
    family: Some(&TINKER_TOOLS_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new TinkerTools()` / `new TinkerTools(uses)`.
pub fn new_tinker_tools(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &TINKER_TOOLS, 0x1EB8, u),
        None => tool::new_tool_random(w, &TINKER_TOOLS, 0x1EB8),
    }
}

/// `Server.Items.TinkersTools`.
pub static TINKERS_TOOLS_CLASS: ToolClass = ToolClass {
    craft_system: "Tinkering",
    ..BASE_TOOL_CLASS
};
pub static TINKERS_TOOLS: ItemType = ItemType {
    full_name: "Server.Items.TinkersTools",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_tinkers_tools(w, None)),
    family: Some(&TINKERS_TOOLS_CLASS),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new TinkersTools()` / `new TinkersTools(uses)`.
pub fn new_tinkers_tools(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &TINKERS_TOOLS, 0x1EBC, u),
        None => tool::new_tool_random(w, &TINKERS_TOOLS, 0x1EBC),
    }
}

/// `Server.Items.Tongs`.
pub static TONGS_CLASS: ToolClass = ToolClass {
    craft_system: "Blacksmithy",
    ..BASE_TOOL_CLASS
};
pub static TONGS: ItemType = ItemType {
    full_name: "Server.Items.Tongs",
    parent: Some(&BASE_TOOL),
    construct: Some(|w| new_tongs(w, None)),
    family: Some(&TONGS_CLASS),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_TOOL
};
/// `new Tongs()` / `new Tongs(uses)`.
pub fn new_tongs(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    match uses {
        Some(u) => tool::new_tool(w, &TONGS, 0xFBB, u),
        None => tool::new_tool_random(w, &TONGS, 0xFBB),
    }
}

/// `Server.Items.Shovel`.
pub static SHOVEL_CLASS: ToolClass = ToolClass {
    harvest_system: "Mining",
    ..BASE_HARVEST_TOOL_CLASS
};
pub static SHOVEL: ItemType = ItemType {
    full_name: "Server.Items.Shovel",
    parent: Some(&BASE_HARVEST_TOOL),
    construct: Some(|w| new_shovel(w, None)),
    family: Some(&SHOVEL_CLASS),
    default_weight: |_, _| 5.0,
    serialize: |w, id, out| {
        (BASE_HARVEST_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_HARVEST_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_HARVEST_TOOL
};
/// `new Shovel()` / `new Shovel(uses)`.
pub fn new_shovel(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    tool::new_harvest_tool(w, &SHOVEL, 0xF39, uses.unwrap_or(50))
}

/// `Server.Items.SturdyShovel`.
pub static STURDY_SHOVEL_CLASS: ToolClass = ToolClass {
    harvest_system: "Mining",
    ..BASE_HARVEST_TOOL_CLASS
};
pub static STURDY_SHOVEL: ItemType = ItemType {
    full_name: "Server.Items.SturdyShovel",
    parent: Some(&BASE_HARVEST_TOOL),
    construct: Some(|w| new_sturdy_shovel(w, None)),
    family: Some(&STURDY_SHOVEL_CLASS),
    default_weight: |_, _| 5.0,
    label_number: |_, _| 1045125,
    serialize: |w, id, out| {
        (BASE_HARVEST_TOOL.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_HARVEST_TOOL.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_HARVEST_TOOL
};
/// `new SturdyShovel()` / `new SturdyShovel(uses)`.
pub fn new_sturdy_shovel(w: &mut muo_server::World, uses: Option<i32>) -> muo_server::ids::ItemId {
    let id = tool::new_harvest_tool(w, &STURDY_SHOVEL, 0xF39, uses.unwrap_or(180));
    let ty = &STURDY_SHOVEL;
    (ty.set_hue)(w, id, 0x973);
    id
}

/// Every tool of this file.
pub static ALL: &[&ItemType] = &[
    &BLOWPIPE,
    &DOVETAIL_SAW,
    &DRAW_KNIFE,
    &FLETCHER_TOOLS,
    &FLOUR_SIFTER,
    &FROE,
    &HAMMER,
    &INSHAVE,
    &JOINTING_PLANE,
    &MALLET_AND_CHISEL,
    &MAPMAKERS_PEN,
    &MORTAR_PESTLE,
    &MOULDING_PLANE,
    &NAILS,
    &ROLLING_PIN,
    &SAW,
    &SCORP,
    &SCRIBES_PEN,
    &SEWING_KIT,
    &SKILLET,
    &SLEDGE_HAMMER,
    &SMITH_HAMMER,
    &SMOOTHING_PLANE,
    &TINKER_TOOLS,
    &TINKERS_TOOLS,
    &TONGS,
    &SHOVEL,
    &STURDY_SHOVEL,
];

pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
}
