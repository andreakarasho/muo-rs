//! `MLQuestPersistence` and the `Serialize`/`Deserialize` methods of MLQuestContext, MLQuestInstance,
//! MLQuest and BaseObjectiveInstance. The system's whole state is saved in one internal item.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Serial};
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::serialize::write_mobile;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::serialization_ext::ReadEntityExt;
use muo_server::world::World;

use super::engine;
use super::model::*;
use super::quester::Quester;

/// `MLQuestPersistence`: not `[Constructible]`; `EnsureExistence` makes the one instance.
pub static ML_QUEST_PERSISTENCE: ItemType = ItemType {
    full_name: "Server.Engines.MLQuests.MLQuestPersistence",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_name: |_, _| Some("ML quests persistence - Internal".to_string()),
    serialize,
    deserialize,
    ..ITEM_BASE
};

/// `MLQuestPersistence.EnsureExistence()`.
pub fn ensure_existence(w: &mut World) {
    if state(w).persistence.is_some_and(|i| w.item(i).is_some()) {
        return;
    }
    let id = ops::new_item_with_id(w, &ML_QUEST_PERSISTENCE, 1);
    ops::set_movable(w, id, false);
    state(w).persistence = Some(id);
}

/// `MLQuestSystem.WriteQuestRef(writer, quest)`.
fn write_quest_ref(w: &World, out: &mut dyn GenericWriter, quest: Option<QuestId>) {
    let name = quest
        .map(|q| engine::quest(w, q))
        .filter(|q| q.save_enabled)
        .map(|q| q.class.full_name);
    out.write_opt_string(name);
}

/// `MLQuestSystem.ReadQuestRef(reader)`.
fn read_quest_ref(w: &World, r: &mut dyn GenericReader) -> Result<Option<QuestId>> {
    Ok(r.read_string()?
        .and_then(|name| engine::find_quest_by_full_name(w, &name)))
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);

    out.write_i32(3); // version
    let s = state_ref(w);
    let contexts: Vec<&Context> = s.map(|s| s.contexts.values().collect()).unwrap_or_default();
    out.write_i32(contexts.len() as i32);

    for context in contexts {
        serialize_context(w, context, out);
    }

    let quests = s.map_or(0, |s| s.quests.len());
    out.write_i32(quests as i32);

    // `MLQuestSystem.Quests.Values`: in registration order.
    for i in 0..quests {
        write_quest_ref(w, out, Some(QuestId(i)));
        out.write_i32(engine::quest(w, QuestId(i)).class.version);
    }
}

/// `MLQuestContext.Serialize`.
fn serialize_context(w: &World, c: &Context, out: &mut dyn GenericWriter) {
    // Version info is written in MLQuestPersistence.Serialize
    write_mobile(w, out, Some(c.owner));
    out.write_i32(c.quest_instances.len() as i32);

    for &inst in &c.quest_instances {
        serialize_instance(w, inst, out);
    }

    out.write_i32(c.done_quests.len() as i32);

    for info in &c.done_quests {
        write_quest_ref(w, out, Some(info.quest));
        out.write_date_time_ticks(info.next_available.ticks());
    }

    out.write_i32(c.chain_offers.len() as i32);

    for &quest in &c.chain_offers {
        write_quest_ref(w, out, Some(quest));
    }

    out.write_encoded_int(c.flags);
}

/// `MLQuestInstance.Serialize`.
fn serialize_instance(w: &World, inst: InstanceId, out: &mut dyn GenericWriter) {
    let Some(i) = engine::instance(w, inst) else {
        // Cannot happen: a context only lists live instances.
        write_quest_ref(w, out, None);
        out.write_serial(Serial::MINUS_ONE);
        out.write_bool(false);
        out.write_i32(0);
        return;
    };

    write_quest_ref(w, out, Some(i.quest));

    let quester = i
        .quester
        .filter(|q| !q.deleted(w))
        .and_then(|q| q.serial(w))
        .unwrap_or(Serial::MINUS_ONE);
    out.write_serial(quester);

    out.write_bool(i.claim_reward());
    out.write_i32(i.objectives.len() as i32);

    for o in &i.objectives {
        // BaseObjectiveInstance.Serialize
        if o.is_timed() {
            out.write_bool(true);
            out.write_anchored_time_ticks(o.end_time.ticks());
        } else {
            out.write_bool(false);
        }

        // For type checks on deserialization
        // (This way quest objectives can be changed without breaking serialization)
        match &o.state {
            ObjectiveState::Escort(e) => {
                out.write_u8(1);
                out.write_bool(e.has_completed);
            }
            ObjectiveState::Kill { slain } => {
                out.write_u8(2);
                out.write_i32(*slain);
            }
            ObjectiveState::Deliver { has_completed } => {
                out.write_u8(3);
                out.write_bool(*has_completed);
            }
            ObjectiveState::Collect | ObjectiveState::GainSkill => out.write_u8(0),
        }
    }
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    state(w).persistence = Some(id);

    let version = r.read_i32()?;
    let contexts = r.read_i32()?;

    for _ in 0..contexts {
        deserialize_context(w, r, version)?;
    }

    let quests = r.read_i32()?;

    for _ in 0..quests {
        // MLQuest.Deserialize
        let quest = read_quest_ref(w, r)?;
        let old_version = r.read_i32()?;

        let Some(q) = quest else { continue }; // not saved or no longer exists
        if let Some(f) = engine::quest(w, q).class.refresh {
            f(w, old_version);
        }
        state(w).quests[q.0].deserialized = true;
    }

    Ok(())
}

/// `new MLQuestContext(reader, version)` and its insertion into `Contexts`.
fn deserialize_context(w: &mut World, r: &mut dyn GenericReader, version: i32) -> Result<()> {
    let owner: Option<MobileId> = r
        .read_mobile(w)?
        .filter(|&m| muo_content::player_mobile::data(w, m).is_some());
    let mut context = owner.map(Context::new);

    let instances = r.read_i32()?;

    for _ in 0..instances {
        let instance = deserialize_instance(w, r, version, owner)?;

        if let (Some(i), Some(c)) = (instance, context.as_mut()) {
            c.quest_instances.push(i);
        }
    }

    let done_quests = r.read_i32()?;

    for _ in 0..done_quests {
        let quest = read_quest_ref(w, r)?;
        let next_available = DateTime::from_ticks(r.read_date_time_ticks()?);

        if let (Some(q), Some(c)) = (quest, context.as_mut())
            && engine::quest(w, q).record_completion()
        {
            c.done_quests.push(DoneQuest {
                quest: q,
                next_available,
            });
        }
    }

    let chain_offers = r.read_i32()?;

    for _ in 0..chain_offers {
        let quest = read_quest_ref(w, r)?;

        if let (Some(q), Some(c)) = (quest, context.as_mut())
            && engine::quest(w, q).is_chain_triggered()
        {
            c.chain_offers.push(q);
        }
    }

    let flags = r.read_encoded_int()?;
    if let Some(c) = context.as_mut() {
        c.flags = flags;
    }

    if let Some(c) = context {
        state(w).contexts.insert(c.owner, c);
    }
    Ok(())
}

/// `MLQuestInstance.Deserialize(reader, version, pm)`.
fn deserialize_instance(
    w: &mut World,
    r: &mut dyn GenericReader,
    version: i32,
    pm: Option<MobileId>,
) -> Result<Option<InstanceId>> {
    let quest = read_quest_ref(w, r)?;

    // TODO: Serialize quester TYPE too, the quest giver reference then becomes optional (only for escorts)
    let quester = r.read_entity(w)?.and_then(Quester::from_entity);

    let claim_reward = r.read_bool()?;
    let objectives = r.read_i32()?;

    let instance = match (quest, quester, pm) {
        (Some(q), Some(qr), Some(pm)) => {
            let inst = engine::create_instance(w, q, Some(qr), pm);
            if let Some(i) = state(w).instances.get_mut(&inst) {
                i.set_flag(iflag::CLAIM_REWARD, claim_reward);
            }
            Some(inst)
        }
        _ => None,
    };

    for idx in 0..objectives.max(0) as usize {
        deserialize_objective(w, r, version, instance, idx)?;
    }

    if let Some(i) = instance {
        engine::instance_slice(w, i);
    }

    Ok(instance)
}

/// `BaseObjectiveInstance.Deserialize(reader, version, objInstance)`.
fn deserialize_objective(
    w: &mut World,
    r: &mut dyn GenericReader,
    version: i32,
    instance: Option<InstanceId>,
    idx: usize,
) -> Result<()> {
    let end_time = if r.read_bool()? {
        let ticks = if version >= 3 {
            r.read_anchored_time_ticks()?
        } else {
            r.read_delta_time_ticks(w.now.ticks())?
        };
        Some(DateTime::from_ticks(ticks))
    } else {
        None
    };

    let target = instance
        .and_then(|i| state(w).instances.get_mut(&i))
        .and_then(|i| i.objectives.get_mut(idx));
    let mut target = target;
    if let (Some(t), Some(e)) = (target.as_deref_mut(), end_time) {
        t.end_time = e;
    }

    match r.read_u8()? {
        1 => {
            let completed = r.read_bool()?;
            if let Some(ObjectiveState::Escort(e)) = target.map(|t| &mut t.state) {
                e.has_completed = completed;
            }
        }
        2 => {
            let slain_n = r.read_i32()?;
            if let Some(ObjectiveState::Kill { slain }) = target.map(|t| &mut t.state) {
                *slain = slain_n;
            }
        }
        3 => {
            let completed = r.read_bool()?;
            if let Some(ObjectiveState::Deliver { has_completed }) = target.map(|t| &mut t.state) {
                *has_completed = completed;
            }
        }
        _ => {}
    }
    Ok(())
}
