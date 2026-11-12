//! `[CommandProperty]` tables written by hand: the `TextDefinition` members (a `Kind::TextDef` whose
//! `Number` / `String` read the parent value) and what `props_gen.rs` cannot say.

use muo_content::props::{self, PropDef, Value};
use muo_content::text_def_prop;
use muo_io::TextDefinition;
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

const GM: AccessLevel = AccessLevel::GameMaster;

fn td(v: &Value) -> Option<TextDefinition> {
    match v {
        Value::TextDef(t) => Some(t.clone()),
        _ => None,
    }
}

fn item(e: EntityId) -> Option<muo_server::ids::ItemId> {
    match e {
        EntityId::Item(i) => Some(i),
        EntityId::Mobile(_) => None,
    }
}

static WARNING_ITEM: &[PropDef] = &[text_def_prop!(
    "WarningMessage",
    GM,
    GM,
    |w: &World, e: EntityId| item(e).and_then(|i| crate::misc::gadgets::warning(w, i).warning),
    Some(|w: &mut World, e: EntityId, v: Value| {
        if let Some(i) = item(e)
            && let Some(d) = crate::misc::gadgets::warning_mut(w, i)
        {
            d.warning = td(&v);
        }
        Ok(())
    })
)];

static HINT_ITEM: &[PropDef] = &[text_def_prop!(
    "HintMessage",
    GM,
    GM,
    |w: &World, e: EntityId| item(e).and_then(|i| crate::misc::gadgets::warning(w, i).hint),
    Some(|w: &mut World, e: EntityId, v: Value| {
        if let Some(i) = item(e)
            && let Some(d) = crate::misc::gadgets::warning_mut(w, i)
        {
            d.hint = td(&v);
        }
        Ok(())
    })
)];

static CONFIRMATION_MOONGATE: &[PropDef] = &[text_def_prop!(
    "Message",
    GM,
    GM,
    |w: &World, e: EntityId| item(e)
        .and_then(|i| crate::misc::moongates::confirmation(w, i))
        .and_then(|d| d.message.clone()),
    Some(|w: &mut World, e: EntityId, v: Value| {
        if let Some(i) = item(e) {
            crate::misc::moongates::confirmation_edit(w, i, |d| d.message = td(&v));
        }
        Ok(())
    })
)];

// `[InvalidateProperties]`
static HOLIDAY_BELL: &[PropDef] = &[text_def_prop!(
    "Maker",
    GM,
    GM,
    |w: &World, e: EntityId| item(e).and_then(|i| crate::special::holiday::bell(w, i).maker),
    Some(|w: &mut World, e: EntityId, v: Value| {
        if let Some(i) = item(e) {
            let mut d = crate::special::holiday::bell(w, i);
            d.maker = td(&v);
            crate::special::holiday::set_bell(w, i, d);
            muo_server::item::ops::invalidate_properties(w, i);
        }
        Ok(())
    })
)];

// ---- BasePlayerBB.Greeting (a `PlayerBBMessage` over the board) ----

fn greeting(w: &World, i: muo_server::ids::ItemId) -> Option<crate::misc::player_bb::BbMessage> {
    crate::misc::player_bb::greeting(w, i)
}

fn greeting_mut(
    w: &mut World,
    i: muo_server::ids::ItemId,
) -> Option<&mut crate::misc::player_bb::BbMessage> {
    w.item_mut(i)?
        .ext
        .get_mut::<crate::misc::player_bb::BbData>()?
        .greeting
        .as_mut()
}

/// The n-th message of a board (`Messages[n]`) a "post props" gump shows: `props::sub_key` says which.
fn posted(w: &World, i: muo_server::ids::ItemId) -> Option<crate::misc::player_bb::BbMessage> {
    crate::misc::player_bb::data(w, i)?
        .messages
        .get(props::sub_key(w))
        .cloned()
}

fn posted_mut(
    w: &mut World,
    i: muo_server::ids::ItemId,
) -> Option<&mut crate::misc::player_bb::BbMessage> {
    let key = props::sub_key(w);
    w.item_mut(i)?
        .ext
        .get_mut::<crate::misc::player_bb::BbData>()?
        .messages
        .get_mut(key)
}

/// `PlayerBBMessage`'s three `[SerializedCommandProperty]` members over `$get` / `$get_mut` (the greeting, or a post).
macro_rules! bb_message_table {
    ($get:ident, $get_mut:ident) => {
        &[
            muo_content::item_prop!(
                "Time",
                props::Kind::DateTime,
                GM,
                GM,
                |w, i| $get(w, i).map_or(Value::Null, |g| Value::DateTime(g.time)),
                |w, i, v| if let (Some(g), Value::DateTime(t)) = ($get_mut(w, i), v) {
                    g.time = t
                }
            ),
            muo_content::item_prop!(
                "Poster",
                props::kinds::ENTITY,
                GM,
                GM,
                |w, i| $get(w, i).map_or(Value::Null, |g| Value::Entity(
                    g.poster.map(EntityId::Mobile)
                )),
                |w, i, v| if let Some(g) = $get_mut(w, i) {
                    g.poster = v.mobile()
                }
            ),
            muo_content::item_prop!(
                "Message",
                props::kinds::STR,
                GM,
                GM,
                |w, i| $get(w, i)
                    .and_then(|g| g.message)
                    .map_or(Value::Null, Value::Str),
                |w, i, v| if let Some(g) = $get_mut(w, i) {
                    g.message = v.string()
                }
            ),
        ]
    };
}

pub(crate) static PLAYER_BB_MESSAGE: &[PropDef] = bb_message_table!(greeting, greeting_mut);

/// `board.Messages[n]`, shown by `PlayerBBGump`'s "post props" button.
pub(crate) static PLAYER_BB_POST: &[PropDef] = bb_message_table!(posted, posted_mut);

static BASE_PLAYER_BB: &[PropDef] = &[PropDef {
    name: "Greeting",
    kind: props::Kind::Object("PlayerBBMessage", PLAYER_BB_MESSAGE),
    read: GM,
    write: GM,
    get: |w, e| match item(e) {
        Some(i) if greeting(w, i).is_some() => Value::Text("...".to_string()),
        _ => Value::Null,
    },
    set: None,
}];

pub fn configure(w: &mut World) {
    props::add(w, "Server.Items.BasePlayerBB", BASE_PLAYER_BB);
    props::add(w, "Server.Items.WarningItem", WARNING_ITEM);
    props::add(w, "Server.Items.HintItem", HINT_ITEM);
    props::add(
        w,
        "Server.Items.ConfirmationMoongate",
        CONFIRMATION_MOONGATE,
    );
    props::add(w, "Server.Items.HolidayBell", HOLIDAY_BELL);
}

#[cfg(test)]
mod post_props_tests {
    use super::*;
    use crate::misc::player_bb::{BbData, BbMessage};
    use muo_server::item::ITEM_BASE;

    /// "post props" shows `Messages[n]`, which `props::sub_key` picks; the greeting table ignores the key.
    #[test]
    fn the_post_table_reads_the_message_the_gump_was_keyed_to() {
        let mut w = World::new();
        let board = w.new_item(&ITEM_BASE);
        let msg = |s: &str| BbMessage {
            message: Some(s.to_string()),
            ..Default::default()
        };
        w.item_mut(board).unwrap().ext.insert(BbData {
            level: 3,
            title: None,
            greeting: Some(msg("hello")),
            messages: vec![msg("first"), msg("second")],
        });
        let e = EntityId::Item(board);
        let get = |w: &mut World, table: &'static [PropDef]| {
            (table.iter().find(|d| d.name == "Message").unwrap().get)(w, e)
        };
        props::set_sub_key(&mut w, 1);
        assert_eq!(get(&mut w, PLAYER_BB_POST), Value::Str("second".into()));
        props::set_sub_key(&mut w, 0);
        assert_eq!(get(&mut w, PLAYER_BB_POST), Value::Str("first".into()));
        assert_eq!(
            get(&mut w, PLAYER_BB_MESSAGE),
            Value::Str("hello".into()),
            "the greeting ignores the key"
        );
        let set = PLAYER_BB_POST
            .iter()
            .find(|d| d.name == "Message")
            .unwrap()
            .set
            .unwrap();
        props::set_sub_key(&mut w, 1);
        set(&mut w, e, Value::Str("edited".into())).unwrap();
        assert_eq!(get(&mut w, PLAYER_BB_POST), Value::Str("edited".into()));
        props::set_sub_key(&mut w, usize::MAX);
        assert_eq!(get(&mut w, PLAYER_BB_POST), Value::Null);
    }
}
