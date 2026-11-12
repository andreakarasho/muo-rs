//! `[Location`: the coordinates of a target.

use muo_server::commands::CommandEventArgs;
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::target::Targeted;
use muo_server::world::World;

use crate::generic::{BaseCommand, CmdSpec, CommandSupport, Obj, ObjectTypes, Run};

pub struct LocationCommand {
    spec: CmdSpec,
}

impl LocationCommand {
    pub fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Location", "Loc", "Pos"],
                AccessLevel::Counselor,
                CommandSupport::SINGLE | CommandSupport::MULTI,
                ObjectTypes::All,
                "Location [itemIds ...]",
                "Retrieves the positional coordinates of a target. Displaying a message above the item, mobile, or environment. Environment targets will also have a graphic temporarily displayed on the tile. This graphic can be changed to the provided list of itemIds.",
            ),
        }
    }
}

impl Default for LocationCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseCommand for LocationCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let point = match obj {
            Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|x| x.location),
            Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map(|x| x.location),
            Targeted::Land(l) => Some(l.location),
            Targeted::Static(s) => Some(s.location),
        };
        let Some(p) = point else {
            return run.log_failure("That cannot be located.");
        };
        let label = format!("(x:{}, y:{}, z:{})", p.x, p.y, p.z);
        match obj {
            Targeted::Land(_) | Targeted::Static(_) => {
                let graphics: Vec<i32> = if e.arguments.is_empty() {
                    vec![0x17AF, 0x17B0, 0x17B1, 0x17B2]
                } else {
                    e.arguments
                        .iter()
                        .filter_map(|a| a.trim().parse::<i32>().ok())
                        .collect()
                };
                let map = w.mobile(e.mobile).and_then(|m| m.map);
                let item = muo_server::item::effect_item::create(
                    w,
                    p,
                    map,
                    muo_server::item::effect_item::DEFAULT_DURATION,
                );
                for g in graphics {
                    muo_server::effects::send_location_particles(
                        w,
                        EntityId::Item(item),
                        g,
                        10,
                        50,
                        2023,
                    );
                }
                muo_server::item::ops::label_to(w, item, e.mobile, &label);
            }
            Targeted::Entity(EntityId::Mobile(m)) => {
                muo_server::mobile::part2::speech::say_to(w, m, e.mobile, &label)
            }
            Targeted::Entity(EntityId::Item(i)) => {
                muo_server::item::ops::label_to(w, i, e.mobile, &label)
            }
        }
        run.add_response(&format!("Location: {label}"));
    }
}
