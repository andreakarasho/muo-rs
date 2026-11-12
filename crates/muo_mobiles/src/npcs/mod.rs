//! Vendors, townfolk, guards, healers, hireables, familiars and specials: the non-monster NPCs.
//! BaseVendor and the banker statics are `muo_content::vendor`.

pub mod banker;
pub mod familiars;
pub mod guards;
pub mod guildmasters;
pub mod healers;
pub mod hires;
pub mod humans;
pub mod humans_support;
pub mod special;
pub mod townfolk;
pub mod vendors;

/// Configure phase: what the familiars call into `PackAnimal`.
fn configure_pack_animal(w: &mut muo_server::World) {
    use crate::hand::pack_animal as pa;
    let h = w.resources.get_mut::<familiars::PackAnimalHooks>();
    h.combine_backpacks = Some(pa::combine_backpacks);
    h.check_access = Some(pa::check_access);
    h.try_pack_open = Some(pa::try_pack_open);
    h.context_menu_entries = Some(pa::context_menu_entries);
    h.release_warning_gump = Some(release_warning_gump);
}

/// `ReleaseFamiliarWarningGump` of `HordeMinionFamiliar`: its OKAY ends the release.
struct ReleaseFamiliarWarning;

impl muo_gumps::m_z::warning::StaticWarning for ReleaseFamiliarWarning {
    const TYPE_NAME: &'static str = "Server.Mobiles.HordeMinionFamiliar+ReleaseFamiliarWarningGump";
    const WIDTH: i32 = 420;
    const HEIGHT: i32 = 280;
    // If you release your familiar, everything in its backpack will be destroyed! Are you sure ...?
    fn static_localized_content(&self) -> i32 {
        1061672
    }
}

/// `from.SendGump(new ReleaseFamiliarWarningGump(okay => ConfirmRelease_Callback(from, okay)))`.
fn release_warning_gump(
    w: &mut muo_server::World,
    familiar: muo_server::ids::MobileId,
    from: muo_server::ids::MobileId,
) {
    let cb: muo_gumps::m_z::warning::WarningCallback = std::rc::Rc::new(move |w, okay| {
        if okay {
            familiars::end_release(w, familiar, Some(from));
        }
    });
    muo_gumps::m_z::common::send(
        w,
        from,
        muo_gumps::m_z::warning::new_static(ReleaseFamiliarWarning, Some(cb)),
    );
}

pub(crate) fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "PackAnimal.Configure",
        configure_pack_animal,
    );
    vendors::register(types);
    special::register(types);
    townfolk::register(types);
    healers::register(types);
    guards::register(types);
    guildmasters::register(types);
    banker::register(types);
    familiars::register(types);
    humans::register(types);
}

/// Every concrete class of this module (for the tests).
#[cfg(test)]
pub(crate) fn all() -> Vec<&'static muo_server::mobile::MobileType> {
    let mut v = Vec::new();
    v.extend(vendors::all());
    v.extend(special::all());
    v.extend(townfolk::classes::all());
    v.extend(healers::all());
    v.extend(guards::all());
    v.extend(guildmasters::all());
    v.extend(banker::all());
    v.extend(familiars::all());
    v.extend(humans::all());
    v
}

#[cfg(test)]
mod golden_tests;
#[cfg(test)]
mod tests;
