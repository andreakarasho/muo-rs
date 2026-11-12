//! The ML quest system: `MLQuestSystem`, the behavior of `MLQuest`, `MLQuestInstance` and the instance halves of
//! the objectives. The classes are data in [`super::model`]; here are the functions over `&mut World`.

use muo_primitives::dotnet::InvariantCase;
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::enumerable::find_items;
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::{self, SkillName};
use muo_server::timer;
use muo_server::world::World;

use muo_content::hooks::rnd;
use muo_content::player_mobile as pmob;

use super::gumps;
use super::model::*;
use super::quester::*;

/// What the escort objective needs from creatures this crate does not own (`BaseEscortable`, `VirtueSystem`,
/// `HumanInNeed`, `Titles`). An unset hook acts as if the class is absent: not controlled, not being deleted, no award.
#[derive(Default)]
pub struct EscortHooks {
    /// `BaseEscortable.IsBeingDeleted`.
    pub is_being_deleted: Option<fn(&World, MobileId) -> bool>,
    /// `BaseEscortable.IsPrisoner`.
    pub is_prisoner: Option<fn(&World, MobileId) -> bool>,
    /// `BaseEscortable.BeginDelete()`.
    pub begin_delete: Option<fn(&mut World, MobileId)>,
    /// `VirtueSystem.AwardVirtue(pm, VirtueName.Compassion, amount)`.
    pub award_compassion: Option<fn(&mut World, MobileId, i32)>,
}

fn escort_hooks(w: &World) -> Option<&EscortHooks> {
    w.resources.get::<EscortHooks>()
}

/// `BaseEscortable.EscortDelay` while the ML quest system is enabled.
pub const ESCORT_DELAY: TimeSpan = TimeSpan::from_mins(1);
/// `BaseEscortable.AbandonDelay` while the ML quest system is enabled.
pub const ABANDON_DELAY: TimeSpan = TimeSpan::from_secs(100);

// ---- type tests (`IsAssignableFrom`) --------------------------------------------------------------------

/// `typeof(name).IsAssignableFrom(item.GetType())`.
pub fn item_is(w: &World, item: ItemId, name: &str) -> bool {
    let Some(i) = w.item(item) else { return false };
    let mut t = Some(i.ty);
    while let Some(ty) = t {
        if short_name(ty.full_name) == name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `typeof(name).IsAssignableFrom(mobile.GetType())`.
pub fn mobile_is(w: &World, mob: MobileId, name: &str) -> bool {
    let Some(m) = w.mobile(mob) else { return false };
    let mut t = Some(m.ty);
    while let Some(ty) = t {
        if short_name(ty.full_name) == name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `typeof(name).IsAssignableFrom(quester.GetType())`.
fn quester_is(w: &World, q: Quester, name: &str) -> bool {
    match q {
        Quester::Mobile(m) => mobile_is(w, m, name),
        Quester::Item(i) => item_is(w, i, name),
    }
}

pub fn make_item_by_name(w: &mut World, name: &str) -> Option<ItemId> {
    match w.types.find_type_by_name(name, false, false)? {
        muo_server::type_registry::EntityType::Item(t) => (t.construct?)(w).into(),
        muo_server::type_registry::EntityType::Mobile(_) => None,
    }
}

// ---- state access ---------------------------------------------------------------------------------------

fn st(w: &World) -> &MlState {
    static EMPTY: std::sync::OnceLock<MlState> = std::sync::OnceLock::new();
    state_ref(w).unwrap_or_else(|| EMPTY.get_or_init(MlState::default))
}

pub fn quest(w: &World, id: QuestId) -> &MlQuest {
    &st(w).quests[id.0]
}

pub fn instance(w: &World, id: InstanceId) -> Option<&Instance> {
    st(w).instances.get(&id)
}

fn instance_mut(w: &mut World, id: InstanceId) -> Option<&mut Instance> {
    state(w).instances.get_mut(&id)
}

/// `MLQuestSystem.Enabled`.
pub fn enabled(w: &World) -> bool {
    st(w).enabled
}

/// `MLQuestSystem.FindQuest(type)` by short class name.
pub fn find_quest(w: &World, name: &str) -> Option<QuestId> {
    st(w)
        .by_class
        .iter()
        .find(|(k, _)| short_name(k) == name)
        .map(|(_, v)| *v)
}

/// `MLQuestSystem.FindQuest(type)` by full class name.
pub fn find_quest_by_full_name(w: &World, full_name: &str) -> Option<QuestId> {
    st(w).by_class.get(full_name).copied()
}

/// `MLQuestSystem.FindQuest(type)` for a `Type` a save holds as the XXH3 hash of its full name.
pub fn find_quest_by_hash(w: &World, hash: u64) -> Option<QuestId> {
    st(w)
        .by_class
        .iter()
        .find(|(k, _)| muo_primitives::hash::compute_hash64(k) == hash)
        .map(|(_, v)| *v)
}

/// `GetContext`.
pub fn get_context(w: &World, pm: MobileId) -> Option<&Context> {
    st(w).contexts.get(&pm)
}

/// `GetOrCreateContext`.
pub fn get_or_create_context(w: &mut World, pm: MobileId) -> &mut Context {
    state(w)
        .contexts
        .entry(pm)
        .or_insert_with(|| Context::new(pm))
}

impl Context {
    /// `IsFull`.
    pub fn is_full(&self) -> bool {
        self.quest_instances.len() >= MAX_CONCURRENT_QUESTS
    }
}

/// `MLQuestSystem.MaxConcurrentQuests`.
pub const MAX_CONCURRENT_QUESTS: usize = 10;

/// `MLQuestContext.FindInstance(quest)`.
pub fn find_instance(w: &World, pm: MobileId, q: QuestId) -> Option<InstanceId> {
    let ctx = get_context(w, pm)?;
    ctx.quest_instances
        .iter()
        .copied()
        .find(|i| instance(w, *i).is_some_and(|x| x.quest == q))
}

/// `MLQuestContext.IsDoingQuest(quest)`.
pub fn is_doing_quest(w: &World, pm: MobileId, q: QuestId) -> bool {
    find_instance(w, pm, q).is_some()
}

/// `MLQuestContext.HasDoneQuest(quest, out nextAvailable)`.
pub fn has_done_quest(w: &World, pm: MobileId, q: QuestId) -> Option<DateTime> {
    get_context(w, pm)?
        .done_quests
        .iter()
        .find(|d| d.quest == q)
        .map(|d| d.next_available)
}

/// `MLQuestContext.SetDoneQuest(quest, nextAvailable)`.
pub fn set_done_quest(w: &mut World, pm: MobileId, q: QuestId, next_available: DateTime) {
    let ctx = get_or_create_context(w, pm);
    if let Some(d) = ctx.done_quests.iter_mut().find(|d| d.quest == q) {
        d.next_available = next_available;
        return;
    }
    ctx.done_quests.push(DoneQuest {
        quest: q,
        next_available,
    });
}

/// `MLQuestContext.RemoveDoneQuest(quest)`.
pub fn remove_done_quest(w: &mut World, pm: MobileId, q: QuestId) {
    if let Some(ctx) = state(w).contexts.get_mut(&pm) {
        ctx.done_quests.retain(|d| d.quest != q);
    }
}

fn is_alive(w: &World, m: MobileId) -> bool {
    muo_server::mobile::base::alive(w, m)
}

fn msg(w: &mut World, pm: MobileId, number: i32) {
    mm::send_localized_message(w, pm, number, "", 0x3B2);
}

fn msg_args(w: &mut World, pm: MobileId, number: i32, args: &str) {
    mm::send_localized_message(w, pm, number, args, 0x3B2);
}

fn play_sound(w: &mut World, pm: MobileId, sound: i32) {
    muo_server::mobile::part2::effects::play_sound(w, pm, sound);
}

fn send_sound(w: &mut World, pm: MobileId, sound: i32) {
    muo_server::mobile::part2::effects::send_sound(w, pm, sound);
}

fn backpack(w: &World, pm: MobileId) -> Option<ItemId> {
    w.mobile(pm).and_then(|m| m.backpack)
}

// ---- MLQuestSystem ---------------------------------------------------------------------------------------

/// `MLQuestSystem.Register(quest)` for a class, with its quester class names.
pub fn register_quest(w: &mut World, class: &'static QuestClass, questers: &[&str]) -> QuestId {
    let quest = (class.create)(class);
    let s = state(w);
    let id = match s.by_class.get(class.full_name) {
        Some(&id) => {
            s.quests[id.0] = quest;
            id
        }
        None => {
            let id = QuestId(s.quests.len());
            s.quests.push(quest);
            s.by_class.insert(class.full_name.to_owned(), id);
            id
        }
    };
    for q in questers {
        register_quest_giver(w, id, q);
    }
    id
}

/// `RegisterQuestGiver(quest, questerType)`.
pub fn register_quest_giver(w: &mut World, id: QuestId, quester: &str) {
    state(w)
        .quest_givers
        .entry(short_name(quester).lower_inv())
        .or_default()
        .push(id);
}

/// `FindQuest(quester, pm, context, out quest, out entry)`.
fn find_quest_for(
    w: &mut World,
    q: Quester,
    pm: MobileId,
) -> Option<(QuestId, Option<InstanceId>)> {
    let quests = quests_of(w, q);
    let quester_type = q.type_name(w);
    let has_context = get_context(w, pm).is_some();

    // 1. Check quests in progress with this NPC (overriding deliveries is intended)
    if has_context {
        for &quest_entry in &quests {
            if let Some(inst) = find_instance(w, pm, quest_entry) {
                let i = instance(w, inst)?;
                if i.quester == Some(q)
                    || (!quest(w, quest_entry).is_escort() && i.quester_type == quester_type)
                {
                    return Some((quest_entry, Some(inst)));
                }
            }
        }
    }

    // 2. Check deliveries (overriding chain offers is intended)
    if let Some(inst) = handle_delivery(w, pm, q, quester_type) {
        let qid = instance(w, inst)?.quest;
        return Some((qid, Some(inst)));
    }

    // 3. Check chain quest offers
    if let Some(ctx) = get_context(w, pm) {
        for &quest_entry in &quests {
            if quest(w, quest_entry).is_chain_triggered() && ctx.chain_offers.contains(&quest_entry)
            {
                return Some((quest_entry, None));
            }
        }
    }

    // 4. Random quest
    random_starter_quest(w, q, pm).map(|qid| (qid, None))
}

/// `MLQuestSystem.OnDoubleClick(quester, pm)`.
pub fn on_double_click(w: &mut World, q: Quester, pm: MobileId) {
    if q.deleted(w) || !is_alive(w, pm) {
        return;
    }

    let Some((quest_id, entry)) = find_quest_for(w, q, pm) else {
        tell(w, q, pm, 1080107); // I'm sorry, I have nothing for you at this time.
        return;
    };

    if let Some(inst) = entry {
        turn_to_face(w, q, pm);

        let Some(i) = instance(w, inst) else { return };
        if i.failed() {
            return; // Note: OSI sends no gump at all for failed quests, they have to be cancelled in the quest overview
        }
        if i.claim_reward() {
            send_reward_offer(w, inst);
        } else if is_completed(w, inst) {
            send_report_back_gump(w, inst);
        } else {
            send_progress_gump(w, inst);
        }
    } else if can_offer(w, quest_id, q, pm, true) {
        turn_to_face(w, q, pm);
        send_offer(w, quest_id, q, pm);
    }
}

/// `CanMarkQuestItem(pm, item, type)`: `type` is the item's class chain, by item.
pub fn can_mark_quest_item(w: &mut World, pm: MobileId, item: ItemId) -> bool {
    let Some(ctx) = get_context(w, pm) else {
        return false;
    };
    let list = ctx.quest_instances.clone();
    list.iter().any(|&inst| {
        instance(w, inst).is_some_and(|i| !i.claim_reward())
            && instance_allows_quest_item(w, inst, item)
    })
}

/// `OnMarkQuestItem`.
fn on_mark_quest_item(w: &mut World, pm: MobileId, item: ItemId) {
    let Some(ctx) = get_context(w, pm) else {
        return;
    };
    let len = ctx.quest_instances.len();

    // We don't foreach because CheckComplete() can potentially modify the MLQuests list
    for i in (0..len).rev() {
        let Some(&inst) = get_context(w, pm).and_then(|c| c.quest_instances.get(i)) else {
            continue;
        };
        let Some(ins) = instance(w, inst) else {
            continue;
        };
        if ins.claim_reward() {
            continue;
        }
        let n = ins.objectives.len();
        for idx in 0..n {
            let ok = instance(w, inst)
                .and_then(|x| x.objectives.get(idx))
                .is_some_and(|o| !o.expired)
                && objective_allows_quest_item(w, inst, idx, item);
            if ok {
                objective_check_complete(w, inst, idx); // yes, this can happen multiple times (for multiple quests)
                break;
            }
        }
    }
}

/// `MLQuestSystem.MarkQuestItem(pm, item)`.
pub fn mark_quest_item(w: &mut World, pm: MobileId, item: ItemId) -> bool {
    if can_mark_quest_item(w, pm, item) {
        muo_server::item::ops::set_quest_item(w, item, true);
        on_mark_quest_item(w, pm, item);
        return true;
    }
    false
}

/// `MLQuestSystem.HandleSkillGain(pm, skill)`.
pub fn handle_skill_gain(w: &mut World, pm: MobileId, skill: SkillName) {
    let Some(ctx) = get_context(w, pm) else {
        return;
    };
    let len = ctx.quest_instances.len();

    for i in (0..len).rev() {
        let Some(&inst) = get_context(w, pm).and_then(|c| c.quest_instances.get(i)) else {
            continue;
        };
        let Some(ins) = instance(w, inst) else {
            continue;
        };
        if ins.claim_reward() {
            continue;
        }
        let quest_id = ins.quest;
        let n = ins.objectives.len();
        for idx in 0..n {
            let expired = instance(w, inst)
                .and_then(|x| x.objectives.get(idx))
                .is_none_or(|o| o.expired);
            let handles = matches!(
                quest(w, quest_id).objectives.get(idx).map(|o| &o.kind),
                Some(ObjectiveKind::GainSkill { skill: s, .. }) if *s == skill
            );
            if !expired && handles {
                objective_check_complete(w, inst, idx);
                break;
            }
        }
    }
}

/// `MLQuestSystem.HandleKill(pm, mob)`.
pub fn handle_kill(w: &mut World, pm: MobileId, mob: MobileId) {
    let Some(ctx) = get_context(w, pm) else {
        return;
    };
    let len = ctx.quest_instances.len();

    for i in (0..len).rev() {
        let Some(&inst) = get_context(w, pm).and_then(|c| c.quest_instances.get(i)) else {
            continue;
        };
        let Some(ins) = instance(w, inst) else {
            continue;
        };
        if ins.claim_reward() {
            continue;
        }
        let quest_id = ins.quest;
        let n = ins.objectives.len();

        /* A kill only counts for a single objective within a quest,
         * but it can count for multiple quests. This is something not
         * currently observable on OSI, so it is assumed behavior.
         */
        for idx in 0..n {
            let live = instance(w, inst)
                .and_then(|x| x.objectives.get(idx))
                .is_some_and(|o| !o.expired);
            let is_kill = matches!(
                quest(w, quest_id).objectives.get(idx).map(|o| &o.kind),
                Some(ObjectiveKind::Kill { .. })
            );
            if live && is_kill && kill_add(w, inst, idx, mob) {
                objective_check_complete(w, inst, idx);
                break;
            }
        }
    }
}

/// `MLQuestSystem.HandleDelivery(pm, quester, questerType)`.
pub fn handle_delivery(
    w: &mut World,
    pm: MobileId,
    q: Quester,
    _quester_type: Option<&'static str>,
) -> Option<InstanceId> {
    let ctx = get_context(w, pm)?;
    let len = ctx.quest_instances.len();
    let mut deliver_instance: Option<InstanceId> = None;

    for i in (0..len).rev() {
        let Some(&inst) = get_context(w, pm).and_then(|c| c.quest_instances.get(i)) else {
            continue;
        };
        // Do NOT skip quests on ClaimReward, because the quester still needs the quest ref!
        let Some(ins) = instance(w, inst) else {
            continue;
        };
        let quest_id = ins.quest;
        let n = ins.objectives.len();

        // Note: On OSI, expired deliveries can still be completed. Bug?
        for idx in 0..n {
            let expired = instance(w, inst)
                .and_then(|x| x.objectives.get(idx))
                .is_none_or(|o| o.expired);
            let dest = match quest(w, quest_id).objectives.get(idx).map(|o| &o.kind) {
                Some(ObjectiveKind::Deliver { destination, .. }) => Some(*destination),
                _ => None,
            };
            let Some(dest) = dest else { continue };
            if expired {
                continue;
            }
            if quester_is(w, q, dest) {
                let completed = matches!(
                    instance(w, inst)
                        .and_then(|x| x.objectives.get(idx))
                        .map(|o| &o.state),
                    Some(ObjectiveState::Deliver {
                        has_completed: true
                    })
                );
                if !completed {
                    // objective completes only once
                    if let Some(o) = instance_mut(w, inst).and_then(|x| x.objectives.get_mut(idx)) {
                        o.state = ObjectiveState::Deliver {
                            has_completed: true,
                        };
                    }
                    objective_check_complete(w, inst, idx);

                    // The quest is continued with this NPC (important for chains)
                    set_instance_quester(w, inst, Some(q));
                }

                deliver_instance.get_or_insert(inst);

                break; // don't return, we may have to complete more deliveries
            }
        }
    }

    deliver_instance
}

/// `MLQuestSystem.HandleDeath(pm)`.
pub fn handle_death(w: &mut World, pm: MobileId) {
    let Some(ctx) = get_context(w, pm) else {
        return;
    };
    let n = ctx.quest_instances.len();
    for i in (0..n).rev() {
        if let Some(&inst) = get_context(w, pm).and_then(|c| c.quest_instances.get(i)) {
            instance_on_player_death(w, inst);
        }
    }
}

/// `MLQuestSystem.HandleDeletion(pm)`.
pub fn handle_player_deletion(w: &mut World, pm: MobileId) {
    let Some(ctx) = get_context(w, pm) else {
        return;
    };
    let n = ctx.quest_instances.len();
    for i in (0..n).rev() {
        if let Some(&inst) = get_context(w, pm).and_then(|c| c.quest_instances.get(i)) {
            instance_remove(w, inst);
        }
    }
    state(w).contexts.shift_remove(&pm);
}

/// `MLQuestSystem.HandleDeletion(quester)`.
pub fn handle_quester_deletion(w: &mut World, q: Quester) {
    for qid in quests_of(w, q) {
        let list = quest(w, qid).instances.clone();
        for inst in list.into_iter().rev() {
            if instance(w, inst).is_some_and(|i| i.quester == Some(q)) {
                instance_on_quester_deleted(w, inst);
            }
        }
    }
}

/// `MLQuestSystem.QuestGumpRequest(m)`.
pub fn quest_gump_request(w: &mut World, m: MobileId) {
    if !enabled(w) || pmob::data(w, m).is_none() {
        return;
    }
    gumps::display_quest_log(w, m);
}

/// `RandomStarterQuest`.
pub fn random_starter_quest(w: &mut World, q: Quester, pm: MobileId) -> Option<QuestId> {
    let quests = quests_of(w, q);
    if quests.is_empty() {
        return None;
    }

    let mut pool: Vec<QuestId> = Vec::new();
    let mut fallback: Option<QuestId> = None;

    for qid in quests {
        if quest(w, qid).is_chain_triggered() || is_doing_quest(w, pm, qid) {
            continue;
        }

        /*
         * Save first quest that reaches the CanOffer call.
         * If no quests are valid at all, return this quest for displaying the CanOffer error message.
         */
        fallback.get_or_insert(qid);

        if can_offer(w, qid, q, pm, false) {
            pool.push(qid);
        }
    }

    if pool.is_empty() {
        fallback
    } else {
        let i = rnd::below(w, pool.len() as i32) as usize;
        Some(pool[i])
    }
}

// ---- MLQuest ---------------------------------------------------------------------------------------------

/// `MLQuest.CanOffer(quester, pm, context, message)`.
pub fn can_offer(w: &mut World, qid: QuestId, q: Quester, pm: MobileId, message: bool) -> bool {
    if let Some(f) = quest(w, qid).class.can_offer {
        return f(w, qid, q, pm, message);
    }
    base_can_offer(w, qid, q, pm, message)
}

/// The body of `MLQuest.CanOffer`.
pub fn base_can_offer(
    w: &mut World,
    qid: QuestId,
    q: Quester,
    pm: MobileId,
    message: bool,
) -> bool {
    if !quest(w, qid).activated || q.deleted(w) {
        return false;
    }

    if let Some(ctx) = get_context(w, pm) {
        if ctx.is_full() {
            if message {
                tell(w, q, pm, 1080107); // I'm sorry, I have nothing for you at this time.
            }
            return false;
        }

        let mut check_quest = Some(qid);

        while let Some(cq) = check_quest {
            if let Some(next_available) = has_done_quest(w, pm, cq) {
                if quest(w, cq).one_time_only {
                    if message {
                        tell(w, q, pm, 1075454); // I cannot offer you the quest again.
                    }
                    return false;
                }

                if next_available > w.now {
                    if message {
                        // I'm sorry, but I don't have anything else for you right now.
                        // Could you check back with me in a few minutes?
                        tell(w, q, pm, 1075575);
                    }
                    return false;
                }
            }

            check_quest = match quest(w, cq).class.next_quest {
                None => None,
                Some(n) => find_quest(w, n),
            };
        }
    }

    let n = quest(w, qid).objectives.len();
    for idx in 0..n {
        if !objective_can_offer(w, qid, idx, q, pm, message) {
            return false;
        }
    }

    true
}

/// `MLQuest.SendOffer`.
pub fn send_offer(w: &mut World, qid: QuestId, q: Quester, pm: MobileId) {
    gumps::display_offer(w, pm, qid, q);
}

/// `MLQuest.OnAccept(quester, pm)`.
pub fn on_accept(w: &mut World, qid: QuestId, q: Quester, pm: MobileId) {
    if !can_offer(w, qid, q, pm, true) {
        return;
    }

    let inst = create_instance(w, qid, Some(q), pm);

    msg(w, pm, 1049019); // You have accepted the Quest.
    send_sound(w, pm, 0x2E7); // private sound

    if let Some(f) = quest(w, qid).class.on_accepted {
        f(w, inst);
    }

    let n = instance(w, inst).map_or(0, |i| i.objectives.len());
    for idx in 0..n {
        objective_on_quest_accepted(w, inst, idx);
    }
}

/// `MLQuest.OnRefuse`.
pub fn on_refuse(w: &mut World, qid: QuestId, pm: MobileId) {
    let text = quest(w, qid).refusal_message.clone();
    gumps::display_conversation(w, pm, qid, text);
}

/// `MLQuest.GetRewards(instance)`: the class's, else the base.
pub fn get_rewards(w: &mut World, inst: InstanceId) {
    let Some(qid) = instance(w, inst).map(|i| i.quest) else {
        return;
    };
    match quest(w, qid).class.get_rewards {
        Some(f) => f(w, inst),
        None => base_get_rewards(w, inst),
    }
}

/// The body of `MLQuest.GetRewards`.
pub fn base_get_rewards(w: &mut World, inst: InstanceId) {
    send_reward_gump(w, inst);
}

/// `BaseEscort.GetRewards`: `AwardHumanInNeed` then the base.
pub fn escort_get_rewards(w: &mut World, inst: InstanceId) {
    let Some(i) = instance(w, inst) else { return };
    let (qid, player) = (i.quest, i.player);
    if quest(w, qid).class.award_human_in_need.unwrap_or(true) {
        super::defs::special::human_in_need_award_to(w, player);
    }
    base_get_rewards(w, inst);
}

/// `MLQuest.GetRestartDelay()`.
fn restart_delay(w: &mut World, qid: QuestId) -> TimeSpan {
    match quest(w, qid).class.get_restart_delay {
        Some(f) => f(w),
        None => TimeSpan::from_secs(i64::from(rnd::from(w, 1, 5)) * 30),
    }
}

/// `MLQuest.CreateInstance(quester, pm)`.
pub fn create_instance(
    w: &mut World,
    qid: QuestId,
    quester: Option<Quester>,
    pm: MobileId,
) -> InstanceId {
    let now = w.now;
    let quester_type = quester.and_then(|q| q.type_name(w));
    let id = {
        let s = state(w);
        s.next_instance += 1;
        InstanceId(s.next_instance)
    };

    let defs = quest(w, qid).objectives.clone();
    let mut objectives = Vec::with_capacity(defs.len());
    let mut timed = false;
    for def in &defs {
        let o = new_objective_instance(w, id, def, quester);
        if o.is_timed() {
            timed = true;
        }
        objectives.push(o);
    }

    state(w).instances.insert(
        id,
        Instance {
            quest: qid,
            quester,
            quester_type,
            player: pm,
            accepted: now,
            flags: 0,
            objectives,
            timer: Default::default(),
        },
    );

    // Register()
    state(w).quests[qid.0].instances.push(id);
    get_or_create_context(w, pm).quest_instances.push(id);

    if timed {
        let h = timer::start(
            w,
            TimeSpan::from_secs(5),
            TimeSpan::from_secs(5),
            0,
            move |w| instance_slice(w, id),
        );
        if let Some(i) = instance_mut(w, id) {
            i.timer = h;
        }
    }

    // The escort objective instance starts its own timer once the instance exists.
    for (idx, def) in defs.iter().enumerate() {
        if matches!(def.kind, ObjectiveKind::Escort { .. }) {
            start_escort_timer(w, id, idx);
        }
    }

    id
}

fn new_objective_instance(
    w: &World,
    _inst: InstanceId,
    def: &Objective,
    quester: Option<Quester>,
) -> ObjectiveInstance {
    let end_time = match def.duration {
        Some(d) => w.now + d,
        None => DateTime::MIN,
    };
    let state = match &def.kind {
        ObjectiveKind::Kill { .. } => ObjectiveState::Kill { slain: 0 },
        ObjectiveKind::Collect { .. } => ObjectiveState::Collect,
        ObjectiveKind::Deliver { .. } => ObjectiveState::Deliver {
            has_completed: false,
        },
        ObjectiveKind::Escort { .. } => {
            let escort = match quester {
                Some(Quester::Mobile(m)) if muo_content::creature::data::cd(w, m).is_some() => {
                    Some(m)
                }
                _ => None,
            };
            ObjectiveState::Escort(EscortState {
                escort,
                has_completed: false,
                last_seen_escorter: w.now,
                timer: Default::default(),
            })
        }
        ObjectiveKind::GainSkill { .. } => ObjectiveState::GainSkill,
    };
    ObjectiveInstance {
        end_time,
        expired: false,
        state,
    }
}

// ---- MLQuestInstance -------------------------------------------------------------------------------------

fn set_instance_quester(w: &mut World, inst: InstanceId, q: Option<Quester>) {
    let t = q.and_then(|q| q.type_name(w));
    if let Some(i) = instance_mut(w, inst) {
        i.quester = q;
        i.quester_type = t;
    }
}

/// `AllowsQuestItem(item, type)`.
pub fn instance_allows_quest_item(w: &World, inst: InstanceId, item: ItemId) -> bool {
    let Some(i) = instance(w, inst) else {
        return false;
    };
    (0..i.objectives.len())
        .any(|idx| !i.objectives[idx].expired && objective_allows_quest_item(w, inst, idx, item))
}

/// `IsCompleted()`.
pub fn is_completed(w: &mut World, inst: InstanceId) -> bool {
    let Some(i) = instance(w, inst) else {
        return false;
    };
    let requires_all = quest(w, i.quest).objective_type == ObjectiveType::All;
    let n = i.objectives.len();

    for idx in 0..n {
        let complete = objective_is_completed(w, inst, idx);
        if complete && !requires_all {
            return true;
        }
        if !complete && requires_all {
            return false;
        }
    }

    requires_all
}

/// `CheckComplete()`.
pub fn instance_check_complete(w: &mut World, inst: InstanceId) {
    if !is_completed(w, inst) {
        return;
    }
    let Some(i) = instance(w, inst) else { return };
    let (player, qid, n) = (i.player, i.quest, i.objectives.len());

    play_sound(w, player, 0x5B5); // public sound

    for idx in 0..n {
        objective_on_quest_completed(w, inst, idx);
    }

    let notice = quest(w, qid).completion_notice.clone();
    muo_content::text::send_message_to_hue(w, Some(&notice), player, 0x23);

    /*
     * Advance to the ClaimReward=true stage if this quest has no
     * completion message to show anyway. This suppresses further
     * triggers of CheckComplete.
     *
     * For quests that require collections, this is done later when
     * the player double clicks the quester.
     */
    // An OnQuestCompleted can potentially have removed this instance already
    let removed = instance(w, inst).is_none_or(|i| i.removed());
    if !removed && skip_report_back(w, inst) && !quest(w, qid).requires_collection() {
        continue_report_back(w, inst, false);
    }
}

/// `SkipReportBack`.
fn skip_report_back(w: &World, inst: InstanceId) -> bool {
    instance(w, inst).is_none_or(|i| quest(w, i.quest).completion_message.is_empty())
}

/// `Fail()`.
pub fn instance_fail(w: &mut World, inst: InstanceId) {
    if let Some(i) = instance_mut(w, inst) {
        i.set_flag(iflag::FAILED, true);
    }
}

/// `Slice()`.
pub fn instance_slice(w: &mut World, inst: InstanceId) {
    let Some(i) = instance(w, inst) else { return };
    if i.claim_reward() || i.removed() {
        stop_instance_timer(w, inst);
        return;
    }
    let (qid, player, n) = (i.quest, i.player, i.objectives.len());

    let mut has_any_fails = false;
    let mut has_any_left = false;

    for idx in 0..n {
        let Some(o) = instance(w, inst).and_then(|i| i.objectives.get(idx)) else {
            continue;
        };
        if o.expired {
            continue;
        }
        if o.is_timed() && o.end_time <= w.now {
            msg(w, player, 1072258); // You failed to complete an objective in time!

            if let Some(o) = instance_mut(w, inst).and_then(|i| i.objectives.get_mut(idx)) {
                o.expired = true;
            }
            objective_on_expire(w, inst, idx);

            has_any_fails = true;
        } else {
            has_any_left = true;
        }
    }

    if (quest(w, qid).objective_type == ObjectiveType::All && has_any_fails) || !has_any_left {
        instance_fail(w, inst);
    }

    if !has_any_left {
        stop_instance_timer(w, inst);
    }
}

fn stop_instance_timer(w: &mut World, inst: InstanceId) {
    let mut h = instance(w, inst).map(|i| i.timer).unwrap_or_default();
    h.cancel(w);
    if let Some(i) = instance_mut(w, inst) {
        i.timer = h;
    }
}

/// `SendProgressGump()`.
pub fn send_progress_gump(w: &mut World, inst: InstanceId) {
    let Some(i) = instance(w, inst) else { return };
    let (player, qid) = (i.player, i.quest);
    let text = quest(w, qid).in_progress_message.clone();
    gumps::display_conversation(w, player, qid, text);
}

/// `SendRewardOffer()`.
pub fn send_reward_offer(w: &mut World, inst: InstanceId) {
    get_rewards(w, inst);
}

/// `SendRewardGump()`.
pub fn send_reward_gump(w: &mut World, inst: InstanceId) {
    let Some(i) = instance(w, inst) else { return };
    let (qid, player, quester) = (i.quest, i.player, i.quester);
    let next_quest_type = quest(w, qid).class.next_quest;

    if let Some(next) = next_quest_type {
        claim_rewards(w, inst); // skip reward gump

        if instance(w, inst).is_some_and(|i| i.removed()) {
            // rewards were claimed successfully
            if let (Some(nq), Some(q)) = (find_quest(w, next), quester) {
                send_offer(w, nq, q, player);
            }
        }
    } else {
        gumps::display_reward(w, player, inst);
    }
}

/// `SendReportBackGump()`.
pub fn send_report_back_gump(w: &mut World, inst: InstanceId) {
    if skip_report_back(w, inst) {
        continue_report_back(w, inst, true); // skip ahead
    } else {
        gumps::display_report_back(w, inst);
    }
}

/// `ContinueReportBack(sendRewardGump)`.
pub fn continue_report_back(w: &mut World, inst: InstanceId, send_reward_gump: bool) {
    let Some(i) = instance(w, inst) else { return };
    let (qid, player, n) = (i.quest, i.player, i.objectives.len());

    // There is a backpack check here on OSI for the rewards as well (even though it's not needed...)

    if quest(w, qid).objective_type == ObjectiveType::All {
        // TODO: 1115877 - You no longer have the required items to complete this quest.
        for idx in 0..n {
            if !objective_is_completed(w, inst, idx) {
                return;
            }
        }

        for idx in 0..n {
            if !objective_on_before_claim_reward(w, inst, idx) {
                return;
            }
        }

        for idx in 0..n {
            objective_on_claim_reward(w, inst, idx);
        }
    } else {
        /* The following behavior is unverified, as OSI (currently) has no collect quest requiring
         * only one objective to be completed. It is assumed that only one objective is claimed
         * (the first completed one), even when multiple are complete.
         */
        let mut complete = false;

        for idx in 0..n {
            if objective_is_completed(w, inst, idx) {
                if objective_on_before_claim_reward(w, inst, idx) {
                    complete = true;
                    objective_on_claim_reward(w, inst, idx);
                }
                break;
            }
        }

        if !complete {
            return;
        }
    }

    if let Some(i) = instance_mut(w, inst) {
        i.set_flag(iflag::CLAIM_REWARD, true);
    }

    if quest(w, qid).has_restart_delay {
        let d = restart_delay(w, qid);
        // `Core.Now + d` throws when it overflows: the rest of the method doesn't run.
        let Some(next) = w.now.checked_add(d) else {
            return;
        };
        set_done_quest(w, player, qid, next);
    }

    // This is correct for ObjectiveType.Any as well
    for idx in 0..n {
        objective_on_after_claim_reward(w, inst, idx);
    }

    if send_reward_gump {
        send_reward_offer(w, inst);
    }
}

/// `ClaimRewards()`.
pub fn claim_rewards(w: &mut World, inst: InstanceId) {
    let Some(i) = instance(w, inst) else { return };
    let (qid, player, n) = (i.quest, i.player, i.objectives.len());
    let player_gone = w.mobile(player).is_none_or(|m| m.deleted);
    if player_gone || !i.claim_reward() || i.removed() {
        return;
    }

    let mut rewards: Vec<ItemId> = Vec::new();

    for reward in quest(w, qid).rewards.clone() {
        add_reward_items(w, &reward, &mut rewards);
    }

    if !rewards.is_empty() {
        // On OSI a more naive method of checking is used.
        // For containers, only the actual container item counts.
        let mut can_fit = true;

        for &item in &rewards {
            if !muo_server::mobile::part2::items::add_to_backpack(w, player, item) {
                can_fit = false;
                break;
            }
        }

        if !can_fit {
            for &item in &rewards {
                muo_server::item::base::delete(w, item);
            }

            // Your backpack is full. You cannot complete the quest and receive your reward.
            msg(w, player, 1078524);
            return;
        }

        for &item in &rewards {
            let Some(it) = w.item(item) else { continue };
            let (name, stackable, amount, ty) = (it.name.clone(), it.stackable(), it.amount, it.ty);
            let reward_name = name.unwrap_or_else(|| format!("#{}", (ty.label_number)(w, item)));

            if stackable {
                // You receive a reward: ~1_QUANTITY~ ~2_ITEM~
                msg_args(w, player, 1115917, &format!("{amount}\t{reward_name}"));
            } else {
                msg_args(w, player, 1074360, &reward_name); // You receive a reward: ~1_REWARD~
            }
        }
    }

    for idx in 0..n {
        objective_on_reward_claimed(w, inst, idx);
    }

    if let Some(f) = quest(w, qid).class.on_reward_claimed {
        f(w, inst);
    }

    let q = quest(w, qid);
    let (record, has_delay, chain, next) = (
        q.record_completion(),
        q.has_restart_delay,
        q.is_chain_triggered(),
        q.class.next_quest,
    );

    if record && !has_delay {
        // Quests with restart delays are logged earlier as per OSI
        set_done_quest(w, player, qid, DateTime::MIN);
    }

    if chain && let Some(ctx) = state(w).contexts.get_mut(&player) {
        ctx.chain_offers.retain(|c| *c != qid);
    }

    if let Some(next) = next
        && let Some(nq) = find_quest(w, next)
    {
        let ctx = get_or_create_context(w, player);
        if !ctx.chain_offers.contains(&nq) {
            ctx.chain_offers.push(nq);
        }
    }

    instance_remove(w, inst);
}

/// `ItemReward.AddRewardItems`.
fn add_reward_items(w: &mut World, reward: &Reward, rewards: &mut Vec<ItemId>) {
    let RewardKind::Item {
        ty,
        amount,
        on_create,
    } = &reward.kind
    else {
        return;
    };
    let Some(mut item) = make_item_by_name(w, ty) else {
        return;
    };
    if let Some(f) = on_create {
        f(w, item);
    }

    if w.item(item).is_some_and(|i| i.stackable()) {
        if *amount > 1 {
            muo_server::item::ops::set_amount(w, item, *amount);
        }
        rewards.push(item);
    } else {
        for i in 0..*amount {
            rewards.push(item);

            if i < amount - 1 {
                match make_item_by_name(w, ty) {
                    Some(n) => {
                        item = n;
                        if let Some(f) = on_create {
                            f(w, item);
                        }
                    }
                    None => return,
                }
            }
        }
    }
}

/// `Cancel(removeChain)`.
pub fn instance_cancel(w: &mut World, inst: InstanceId, remove_chain: bool) {
    instance_remove(w, inst);

    let Some(i) = instance(w, inst) else { return };
    let (player, qid, n) = (i.player, i.quest, i.objectives.len());

    send_sound(w, player, 0x5B3); // private sound

    for idx in 0..n {
        objective_on_quest_cancelled(w, inst, idx);
    }

    if let Some(f) = quest(w, qid).class.on_cancel {
        f(w, inst);
    }

    if remove_chain && let Some(ctx) = state(w).contexts.get_mut(&player) {
        ctx.chain_offers.retain(|c| *c != qid);
    }
}

/// `Remove()`: `Unregister()` and `StopTimer()`.
pub fn instance_remove(w: &mut World, inst: InstanceId) {
    let Some(i) = instance(w, inst) else { return };
    let (qid, player) = (i.quest, i.player);
    let s = state(w);
    s.quests[qid.0].instances.retain(|x| *x != inst);
    if let Some(ctx) = s.contexts.get_mut(&player) {
        ctx.quest_instances.retain(|x| *x != inst);
    } else {
        // `PlayerContext` creates the context, which Remove then removes the instance from.
        get_or_create_context(w, player);
    }
    if let Some(i) = instance_mut(w, inst) {
        i.set_flag(iflag::REMOVED, true);
    }
    stop_instance_timer(w, inst);
}

/// `OnQuesterDeleted()`.
fn instance_on_quester_deleted(w: &mut World, inst: InstanceId) {
    let Some(i) = instance(w, inst) else { return };
    let (qid, n) = (i.quest, i.objectives.len());
    for idx in 0..n {
        objective_on_quester_deleted(w, inst, idx);
    }
    if let Some(f) = quest(w, qid).class.on_quester_deleted {
        f(w, inst);
    }
}

/// `OnPlayerDeath()`.
fn instance_on_player_death(w: &mut World, inst: InstanceId) {
    let Some(i) = instance(w, inst) else { return };
    let (qid, n) = (i.quest, i.objectives.len());
    for idx in 0..n {
        objective_on_player_death(w, inst, idx);
    }
    if let Some(f) = quest(w, qid).class.on_player_death {
        f(w, inst);
    }
}

// ---- objectives ------------------------------------------------------------------------------------------

fn objective_def(w: &World, inst: InstanceId, idx: usize) -> Option<Objective> {
    let i = instance(w, inst)?;
    quest(w, i.quest).objectives.get(idx).cloned()
}

/// `BaseObjective.CanOffer(quester, pm, message)`.
fn objective_can_offer(
    w: &mut World,
    qid: QuestId,
    idx: usize,
    q: Quester,
    pm: MobileId,
    message: bool,
) -> bool {
    let Some(def) = quest(w, qid).objectives.get(idx).cloned() else {
        return true;
    };
    match def.kind {
        ObjectiveKind::GainSkill {
            skill,
            threshold_fixed,
            use_real,
            ..
        } => {
            let value = if use_real {
                skills::fixed(w, pm, skill)
            } else {
                skills::base_fixed(w, pm, skill)
            };
            if value >= threshold_fixed {
                if message {
                    tell(w, q, pm, 1077772); // I cannot teach you, for you know all I can teach!
                }
                return false;
            }
            true
        }
        ObjectiveKind::Escort { .. } => {
            if let Quester::Mobile(m) = q {
                let controlled =
                    muo_content::creature::data::cd(w, m).is_some_and(|d| d.controlled);
                let deleting = escort_hooks(w)
                    .and_then(|h| h.is_being_deleted)
                    .is_some_and(|f| f(w, m));
                if controlled || deleting {
                    return false;
                }
            }

            if let Some(ctx) = get_context(w, pm) {
                let list = ctx.quest_instances.clone();
                for inst in list {
                    if instance(w, inst).is_some_and(|i| quest(w, i.quest).is_escort()) {
                        if message {
                            tell(w, q, pm, 500896); // I see you already have an escort.
                        }
                        return false;
                    }
                }
            }

            let last = pmob::data(w, pm).map_or(DateTime::MIN, |d| d.last_escort_time);
            let next_escort = last + ESCORT_DELAY;
            if next_escort > w.now {
                if message {
                    let minutes = (next_escort - w.now).total_minutes().ceil() as i32;

                    if minutes == 1 {
                        tell(w, q, pm, 1042722);
                    } else {
                        // You must rest ~1_minsleft~ minutes before we set out on this journey.
                        tell_args(w, q, pm, 1071195, &minutes.to_string());
                    }
                }
                return false;
            }
            true
        }
        _ => true,
    }
}

/// `BaseObjectiveInstance.AllowsQuestItem(item, type)`.
fn objective_allows_quest_item(w: &World, inst: InstanceId, idx: usize, item: ItemId) -> bool {
    match objective_def(w, inst, idx).map(|o| o.kind) {
        Some(ObjectiveKind::Collect { ty, check_item, .. }) => {
            item_is(w, item, ty) && check_item.is_none_or(|f| f(w, item))
        }
        _ => false,
    }
}

/// The pack items a collect/deliver objective counts: direct children of the backpack.
fn pack_items(
    w: &World,
    pm: MobileId,
    mut pred: impl FnMut(&World, ItemId) -> bool,
) -> Vec<ItemId> {
    match backpack(w, pm) {
        Some(pack) => find_items(w, pack, false, |w, id| pred(w, id)),
        None => Vec::new(),
    }
}

fn collect_total(
    w: &World,
    pm: MobileId,
    ty: &str,
    check_item: Option<fn(&World, ItemId) -> bool>,
) -> i32 {
    pack_items(w, pm, |w, item| {
        item_is(w, item, ty)
            && w.item(item).is_some_and(|i| i.quest_item())
            && check_item.is_none_or(|f| f(w, item))
    })
    .iter()
    .map(|&i| w.item(i).map_or(0, |i| i.amount))
    .sum()
}

fn deliver_total(w: &World, pm: MobileId, ty: &str) -> i32 {
    pack_items(w, pm, |w, item| item_is(w, item, ty))
        .iter()
        .map(|&i| w.item(i).map_or(0, |i| i.amount))
        .sum()
}

/// The progress numbers a gump shows for an objective.
pub fn collect_current_total(w: &World, inst: InstanceId, idx: usize) -> i32 {
    let (Some(i), Some(def)) = (instance(w, inst), objective_def(w, inst, idx)) else {
        return 0;
    };
    match def.kind {
        ObjectiveKind::Collect { ty, check_item, .. } => collect_total(w, i.player, ty, check_item),
        _ => 0,
    }
}

/// `IsCompleted()` of an objective instance.
pub fn objective_is_completed(w: &mut World, inst: InstanceId, idx: usize) -> bool {
    let (Some(i), Some(def)) = (instance(w, inst), objective_def(w, inst, idx)) else {
        return false;
    };
    let player = i.player;
    let Some(o) = i.objectives.get(idx) else {
        return false;
    };
    match (&def.kind, &o.state) {
        (ObjectiveKind::Kill { amount, .. }, ObjectiveState::Kill { slain }) => slain >= amount,
        (
            ObjectiveKind::Collect {
                amount,
                ty,
                check_item,
                ..
            },
            _,
        ) => collect_total(w, player, ty, *check_item) >= *amount,
        (_, ObjectiveState::Deliver { has_completed }) => *has_completed,
        (_, ObjectiveState::Escort(e)) => e.has_completed,
        (
            ObjectiveKind::GainSkill {
                skill,
                threshold_fixed,
                use_real,
                ..
            },
            _,
        ) => {
            let value = if *use_real {
                skills::fixed(w, player, *skill)
            } else {
                skills::base_fixed(w, player, *skill)
            };
            value >= *threshold_fixed
        }
        _ => false,
    }
}

/// `CheckComplete()` of an objective instance.
pub fn objective_check_complete(w: &mut World, inst: InstanceId, idx: usize) {
    if objective_is_completed(w, inst, idx) {
        let Some(p) = instance(w, inst).map(|i| i.player) else {
            return;
        };
        play_sound(w, p, 0x5B6); // public sound
        instance_check_complete(w, inst);
    }
}

/// `KillObjectiveInstance.AddKill(mob, type)`.
fn kill_add(w: &mut World, inst: InstanceId, idx: usize, mob: MobileId) -> bool {
    let Some(ObjectiveKind::Kill {
        amount: desired,
        types,
        area,
        ..
    }) = objective_def(w, inst, idx).map(|o| o.kind)
    else {
        return false;
    };
    let Some(player) = instance(w, inst).map(|i| i.player) else {
        return false;
    };

    for accepted in types {
        if mobile_is(w, mob, accepted) {
            if let Some(area) = &area {
                let region = w.mobile(mob).map(|m| m.region);
                let inside = region
                    .is_some_and(|r| muo_server::region::is_part_of_name(w, r, area.region, false));
                if !inside {
                    return false;
                }
            }

            let mut slain = 0;
            if let Some(ObjectiveState::Kill { slain: s }) = instance_mut(w, inst)
                .and_then(|i| i.objectives.get_mut(idx))
                .map(|o| &mut o.state)
            {
                *s += 1;
                slain = *s;
            }

            if slain >= desired {
                msg(w, player, 1075050); // You have killed all the required quest creatures of this type.
            } else {
                // You have killed a quest creature. ~1_val~ more left.
                msg_args(w, player, 1075051, &(desired - slain).to_string());
            }

            return true;
        }
    }

    false
}

/// `OnQuestAccepted()`.
fn objective_on_quest_accepted(w: &mut World, inst: InstanceId, idx: usize) {
    let (Some(i), Some(def)) = (instance(w, inst), objective_def(w, inst, idx)) else {
        return;
    };
    let (player, quester) = (i.player, i.quester);
    match def.kind {
        ObjectiveKind::Deliver {
            delivery,
            amount,
            spawns_delivery,
            ..
        } => {
            // DeliverObjective.SpawnDelivery(pack)
            let Some(pack) = backpack(w, player) else {
                return;
            };
            if !spawns_delivery {
                return;
            }
            let mut items = Vec::new();
            for _ in 0..amount {
                if let Some(item) = make_item_by_name(w, delivery) {
                    items.push(item);
                    if w.item(item).is_some_and(|i| i.stackable()) && amount > 1 {
                        muo_server::item::ops::set_amount(w, item, amount);
                        break;
                    }
                }
            }
            for item in items {
                // Confirmed: on OSI items are added even if your pack is full
                if let Some(ty) = w.item(pack).map(|p| p.ty) {
                    (ty.drop_item)(w, pack, item);
                }
            }
        }
        ObjectiveKind::Escort { .. } => {
            let now = w.now;
            if let Some(d) = pmob::data_mut(w, player) {
                d.last_escort_time = now;
            }
            if let Some(Quester::Mobile(esc)) = quester {
                begin_follow(w, esc, player);
            }
        }
        ObjectiveKind::GainSkill {
            skill, accelerate, ..
        } => {
            if !accelerate {
                return;
            }
            // TODO: This may interfere with scrolls, or even quests among each other
            let until = w.now + TimeSpan::from_mins(15); // TODO: Is there a max duration?
            if let Some(d) = pmob::data_mut(w, player) {
                d.accelerated_skill = skill;
                d.accelerated_start = until;
            }
        }
        _ => {}
    }
}

/// `OnQuestCancelled()`.
fn objective_on_quest_cancelled(w: &mut World, inst: InstanceId, idx: usize) {
    let (Some(i), Some(def)) = (instance(w, inst), objective_def(w, inst, idx)) else {
        return;
    };
    let player = i.player;
    match def.kind {
        ObjectiveKind::Collect { ty, check_item, .. } => {
            // does another quest still need this item? (OSI just unmarks everything)
            let items = pack_items(w, player, |w, item| {
                item_is(w, item, ty) && w.item(item).is_some_and(|i| i.quest_item())
            });
            for item in items {
                if !can_mark_quest_item(w, player, item) {
                    muo_server::item::ops::set_quest_item(w, item, false);
                }
            }
            let _ = check_item;
        }
        ObjectiveKind::Deliver { .. } => objective_on_claim_reward(w, inst, idx), // same effect
        ObjectiveKind::GainSkill { accelerate, .. } => {
            if !accelerate {
                return;
            }
            let now = w.now;
            if let Some(d) = pmob::data_mut(w, player) {
                d.accelerated_start = now;
            }
            play_sound(w, player, 0x100);
        }
        _ => {}
    }
}

/// `OnQuestCompleted()`.
fn objective_on_quest_completed(w: &mut World, inst: InstanceId, idx: usize) {
    if matches!(
        objective_def(w, inst, idx).map(|o| o.kind),
        Some(ObjectiveKind::GainSkill { .. })
    ) {
        objective_on_quest_cancelled(w, inst, idx);
    }
}

/// `OnBeforeClaimReward()`.
fn objective_on_before_claim_reward(w: &mut World, inst: InstanceId, idx: usize) -> bool {
    let (Some(i), Some(def)) = (instance(w, inst), objective_def(w, inst, idx)) else {
        return true;
    };
    let player = i.player;
    if let ObjectiveKind::Deliver {
        delivery, amount, ..
    } = def.kind
    {
        let total = deliver_total(w, player, delivery);

        if total < amount {
            msg(w, player, 1074861); // You do not have everything you need!
            msg_args(w, player, 1074885, &format!("{total}\t{amount}")); // You have ~1_val~ item(s) but require ~2_val~
            return false;
        }
    }
    true
}

/// `OnClaimReward()`.
fn objective_on_claim_reward(w: &mut World, inst: InstanceId, idx: usize) {
    let (Some(i), Some(def)) = (instance(w, inst), objective_def(w, inst, idx)) else {
        return;
    };
    let player = i.player;
    if backpack(w, player).is_none() {
        return;
    }
    match def.kind {
        ObjectiveKind::Collect { ty, check_item, .. } => {
            // TODO: OSI also counts the item in the cursor?
            let mut left = match def.kind {
                ObjectiveKind::Collect { amount, .. } => amount,
                _ => 0,
            };
            let items = pack_items(w, player, |w, item| item_is(w, item, ty));
            for item in items {
                let Some(it) = w.item(item) else { continue };
                if it.quest_item() && check_item.is_none_or(|f| f(w, item)) {
                    if left == 0 {
                        return;
                    }
                    let amt = it.amount;
                    if amt > left {
                        muo_server::item::base::consume(w, item, left);
                        left = 0;
                    } else {
                        muo_server::item::base::delete(w, item);
                        left -= amt;
                    }
                }
            }
        }
        ObjectiveKind::Deliver {
            delivery, amount, ..
        } => {
            let mut left = amount;
            let items = pack_items(w, player, |w, item| item_is(w, item, delivery));
            for item in items {
                if left == 0 {
                    break;
                }
                let Some(amt) = w.item(item).map(|i| i.amount) else {
                    continue;
                };
                if amt > left {
                    muo_server::item::base::consume(w, item, left);
                    left = 0;
                } else {
                    muo_server::item::base::delete(w, item);
                    left -= amt;
                }
            }
        }
        _ => {}
    }
}

/// `OnAfterClaimReward()`.
fn objective_on_after_claim_reward(w: &mut World, inst: InstanceId, idx: usize) {
    if matches!(
        objective_def(w, inst, idx).map(|o| o.kind),
        Some(ObjectiveKind::Collect { .. })
    ) {
        objective_on_quest_cancelled(w, inst, idx); // same thing, clear other quest items
    }
}

/// `OnRewardClaimed()`: no objective overrides it.
fn objective_on_reward_claimed(_w: &mut World, _inst: InstanceId, _idx: usize) {}

/// `OnQuesterDeleted()`.
fn objective_on_quester_deleted(w: &mut World, inst: InstanceId, idx: usize) {
    if matches!(
        objective_def(w, inst, idx).map(|o| o.kind),
        Some(ObjectiveKind::Escort { .. })
    ) {
        let done = objective_is_completed(w, inst, idx);
        if done || instance(w, inst).is_none_or(|i| i.removed()) {
            return;
        }
        escort_abandon(w, inst, idx);
    }
}

/// `OnPlayerDeath()`.
fn objective_on_player_death(w: &mut World, inst: InstanceId, idx: usize) {
    if matches!(
        objective_def(w, inst, idx).map(|o| o.kind),
        Some(ObjectiveKind::Escort { .. })
    ) {
        // Note: OSI also cancels it when the quest is already complete
        if instance(w, inst).is_none_or(|i| i.removed()) {
            return;
        }
        instance_cancel(w, inst, false);
    }
}

/// `OnExpire()`.
fn objective_on_expire(w: &mut World, inst: InstanceId, idx: usize) {
    let Some(def) = objective_def(w, inst, idx) else {
        return;
    };
    let Some(player) = instance(w, inst).map(|i| i.player) else {
        return;
    };
    match def.kind {
        ObjectiveKind::Collect { .. } => objective_on_quest_cancelled(w, inst, idx), // No message
        ObjectiveKind::Deliver { .. } => {
            objective_on_quest_cancelled(w, inst, idx);
            msg(w, player, 1074813); // You have failed to complete your delivery.
        }
        ObjectiveKind::Escort { .. } => escort_abandon(w, inst, idx),
        _ => {}
    }
}

// ---- escort ----------------------------------------------------------------------------------------------

fn escort_state(w: &World, inst: InstanceId, idx: usize) -> Option<&EscortState> {
    match instance(w, inst)?.objectives.get(idx).map(|o| &o.state) {
        Some(ObjectiveState::Escort(e)) => Some(e),
        _ => None,
    }
}

fn escort_state_mut(w: &mut World, inst: InstanceId, idx: usize) -> Option<&mut EscortState> {
    match instance_mut(w, inst)?
        .objectives
        .get_mut(idx)
        .map(|o| &mut o.state)
    {
        Some(ObjectiveState::Escort(e)) => Some(e),
        _ => None,
    }
}

fn start_escort_timer(w: &mut World, inst: InstanceId, idx: usize) {
    let h = timer::start(
        w,
        TimeSpan::from_secs(5),
        TimeSpan::from_secs(5),
        0,
        move |w| escort_check_destination(w, inst, idx),
    );
    if let Some(e) = escort_state_mut(w, inst, idx) {
        e.timer = h;
    }
}

fn stop_escort_timer(w: &mut World, inst: InstanceId, idx: usize) {
    let mut h = escort_state(w, inst, idx)
        .map(|e| e.timer)
        .unwrap_or_default();
    h.cancel(w);
    if let Some(e) = escort_state_mut(w, inst, idx) {
        e.timer = h;
    }
}

/// `EscortObjectiveInstance.m_Escort`.
fn escort_creature(w: &World, inst: InstanceId, idx: usize) -> Option<MobileId> {
    escort_state(w, inst, idx)?.escort
}

/// `CheckDestination()`.
fn escort_check_destination(w: &mut World, inst: InstanceId, idx: usize) {
    let escort = escort_creature(w, inst, idx);
    let completed = escort_state(w, inst, idx).is_none_or(|e| e.has_completed);
    let Some(esc) = escort.filter(|_| !completed) else {
        // Completed by deserialization
        stop_escort_timer(w, inst, idx);
        return;
    };

    let Some(i) = instance(w, inst) else { return };
    let (player, removed, qid) = (i.player, i.removed(), i.quest);
    let destination = match quest(w, qid).objectives.get(idx).map(|o| &o.kind) {
        Some(ObjectiveKind::Escort { destination }) => destination.clone(),
        _ => None,
    };

    let in_dest = destination.as_ref().is_some_and(|d| {
        w.mobile(esc)
            .is_some_and(|m| muo_server::region::is_part_of_name(w, m.region, d.region, false))
    });

    if removed {
        escort_abandon(w, inst, idx);
    } else if in_dest {
        // We have arrived! I thank thee, ~1_PLAYER_NAME~! I have no further need of thy services. Here is thy pay.
        let name = w
            .mobile(player)
            .and_then(|m| (m.ty.name)(w, player))
            .unwrap_or_default();
        mm::say_number(w, esc, 1042809, &name);

        let young = pmob::young(w, player);
        let in_haven = w.mobile(esc).is_some_and(|m| {
            muo_server::region::is_part_of_name(w, m.region, "Haven Island", false)
        });
        if young || in_haven {
            muo_content::misc::titles::award_fame(w, player, 10, true);
        } else {
            let prisoner = escort_hooks(w)
                .and_then(|h| h.is_prisoner)
                .is_some_and(|f| f(w, esc));
            if let Some(f) = escort_hooks(w).and_then(|h| h.award_compassion) {
                f(w, player, if prisoner { 400 } else { 200 });
            }
        }

        end_follow(w, esc);
        stop_escort_timer(w, inst, idx);

        if let Some(e) = escort_state_mut(w, inst, idx) {
            e.has_completed = true;
        }
        objective_check_complete(w, inst, idx);

        // Auto claim reward
        on_double_click(w, Quester::Mobile(esc), player);
    } else {
        let (pm_map, esc_map) = (
            w.mobile(player).and_then(|m| m.map),
            w.mobile(esc).and_then(|m| m.map),
        );
        let near = w.mobile(esc).map(|m| m.location).is_some_and(|loc| {
            w.mobile(player)
                .is_some_and(|p| in_range_xy(p.location, loc, 30))
        });
        if pm_map != esc_map || !near {
            // TODO: verify range
            let last = escort_state(w, inst, idx).map_or(w.now, |e| e.last_seen_escorter);
            if last + ABANDON_DELAY <= w.now {
                escort_abandon(w, inst, idx);
            }
        } else {
            let now = w.now;
            if let Some(e) = escort_state_mut(w, inst, idx) {
                e.last_seen_escorter = now;
            }
        }
    }
}

fn in_range_xy(a: Point3D, b: Point3D, range: i32) -> bool {
    (a.x - b.x).abs() <= range && (a.y - b.y).abs() <= range
}

/// `BeginFollow(quester, pm)`.
pub fn begin_follow(w: &mut World, quester: MobileId, pm: MobileId) {
    use muo_content::creature::{control, data::cd_mut, enums::OrderType, speeds};
    if let Some(d) = cd_mut(w, quester) {
        d.control_slots = 0;
    }
    control::set_control_master(w, quester, Some(pm));

    control::set_control_order(w, quester, OrderType::Follow, None, false, None);
    if let Some(d) = cd_mut(w, quester) {
        d.control_target = Some(pm);
    }

    if let Some(m) = w.mobile_mut(quester) {
        m.cant_walk = false;
    }

    speeds::set_speed(w, quester, 0.1, 0.2, false);
}

/// `EndFollow(quester)`.
pub fn end_follow(w: &mut World, quester: MobileId) {
    use muo_content::creature::{control, data::cd_mut, enums::OrderType, speeds};
    control::set_control_order(w, quester, OrderType::None, None, false, None);
    if let Some(d) = cd_mut(w, quester) {
        d.control_target = None;
    }

    control::set_control_master(w, quester, None);

    speeds::set_speed(w, quester, 0.1, 0.2, true);

    if let Some(f) = escort_hooks(w).and_then(|h| h.begin_delete) {
        f(w, quester);
    }
}

/// `Abandon()`.
fn escort_abandon(w: &mut World, inst: InstanceId, idx: usize) {
    stop_escort_timer(w, inst, idx);

    let Some(i) = instance(w, inst) else { return };
    let (player, removed) = (i.player, i.removed());
    let esc = escort_creature(w, inst, idx);

    if let Some(esc) = esc.filter(|&e| w.mobile(e).is_some_and(|m| !m.deleted)) {
        if !is_alive(w, player) {
            mm::say_number(w, esc, 500901, ""); // Ack!  My escort has come to haunt me!
        } else {
            mm::say_number(w, esc, 500902, ""); // My escort seems to have abandoned me!
        }

        end_follow(w, esc);
    }

    // Note: this sound is sent twice on OSI (once here and once in Cancel())
    msg(w, player, 1071194); // You have failed your escort quest...

    if !removed {
        instance_cancel(w, inst, false);
    }
}

// ---- CheckShout -----------------------------------------------------------------------------------------

/// `BaseCreature.CheckShout(pm, oldLocation)`; `shout` is the creature class's `Shout(pm)` override.
pub fn check_shout(
    w: &mut World,
    creature: MobileId,
    pm: MobileId,
    old_location: Point3D,
    shout_range: i32,
) {
    let next = st(w)
        .next_shout
        .get(&creature)
        .copied()
        .unwrap_or(DateTime::MIN);
    let hidden = w.mobile(pm).is_none_or(|m| m.hidden);
    if next > w.now || hidden || !is_alive(w, pm) {
        return;
    }

    let Some(cl) = w.mobile(creature).map(|m| m.location) else {
        return;
    };
    let Some(pl) = w.mobile(pm).map(|m| m.location) else {
        return;
    };
    let in_range = |a: Point3D, b: Point3D| in_range_xy(a, b, shout_range);
    if !in_range(pl, cl)
        || in_range(old_location, cl)
        || !muo_server::mobile::base::can_see_mobile(w, creature, pm)
        || !muo_server::mobile::part1::combat::in_los(w, creature, pm)
    {
        return;
    }

    if get_context(w, pm).is_some_and(|c| c.is_full()) {
        return;
    }

    let Some(qid) = random_starter_quest(w, Quester::Mobile(creature), pm) else {
        return;
    };
    if !quest(w, qid).activated || is_doing_quest(w, pm, qid) {
        return;
    }

    if let Some(f) = super::npcs::shout_for(w, creature) {
        f(w, creature, pm);
    }
    let until = w.now + SHOUT_DELAY;
    state(w).next_shout.insert(creature, until);
}

/// `BaseCreature.ShoutDelay`.
const SHOUT_DELAY: TimeSpan = TimeSpan::from_mins(1);
