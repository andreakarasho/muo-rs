//! Values a script keeps across restarts: `store_set(key, value)`, `store_get(key)`, `store_remove(key)` and
//! `store_keys(prefix)`. A value is a number, string, bool, array or map of those (not a mobile or item handle: keep
//! its `serial`). They are saved with the world, in the "ScriptData" persistence, as one JSON object.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::persistence::{GenericPayload, GenericPersistence};
use muo_server::world::World;
use rhai::{Array, Dynamic, Engine, NativeCallContext};
use serde_json::{Map, Value};

use super::api::{R, with_world};

/// The saved values, by key.
#[derive(Default)]
pub struct ScriptStore(pub Map<String, Value>);

pub(crate) fn register(e: &mut Engine) {
    e.register_fn(
        "store_get",
        |ctx: NativeCallContext, key: &str| -> R<Dynamic> {
            with_world(&ctx, |w| {
                w.resources
                    .get::<ScriptStore>()
                    .and_then(|s| s.0.get(key))
                    .and_then(|v| rhai::serde::to_dynamic(v).ok())
                    .unwrap_or(Dynamic::UNIT)
            })
        },
    );
    e.register_fn(
        "store_set",
        |ctx: NativeCallContext, key: &str, value: Dynamic| -> R<()> {
            let v: Value = rhai::serde::from_dynamic(&value).map_err(|e| {
                format!("store_set {key}: only numbers, text, bools, arrays and maps keep ({e})")
            })?;
            with_world(&ctx, |w| {
                w.resources
                    .get_mut::<ScriptStore>()
                    .0
                    .insert(key.to_owned(), v);
            })
        },
    );
    e.register_fn(
        "store_remove",
        |ctx: NativeCallContext, key: &str| -> R<()> {
            with_world(&ctx, |w| {
                w.resources.get_mut::<ScriptStore>().0.remove(key);
            })
        },
    );
    e.register_fn(
        "store_keys",
        |ctx: NativeCallContext, prefix: &str| -> R<Array> {
            with_world(&ctx, |w| {
                w.resources
                    .get::<ScriptStore>()
                    .map_or_else(Array::new, |s| {
                        s.0.keys()
                            .filter(|k| k.starts_with(prefix))
                            .map(|k| k.as_str().into())
                            .collect()
                    })
            })
        },
    );
}

/// The "ScriptData" persistence: the store as one JSON string.
struct ScriptDataPayload;

impl GenericPayload for ScriptDataPayload {
    fn name(&self) -> &str {
        "ScriptData"
    }

    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(0); // version
        let json = w.resources.get::<ScriptStore>().map_or_else(
            || "{}".to_owned(),
            |s| Value::Object(s.0.clone()).to_string(),
        );
        writer.write_opt_string(Some(&json));
    }

    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> Result<()> {
        let _version = reader.read_encoded_int()?;
        let text = reader.read_string()?.unwrap_or_default();
        // a store that does not parse is dropped (and said so) rather than stopping the world from loading
        match serde_json::from_str::<Map<String, Value>>(&text) {
            Ok(m) => w.resources.insert(ScriptStore(m)),
            Err(e) => tracing::error!(
                "ScriptData: the saved script values do not parse, starting empty: {e}"
            ),
        }
        Ok(())
    }
}

/// Registers the persistence (before the world loads).
pub(crate) fn configure(w: &mut World) {
    w.resources.insert(ScriptStore::default());
    w.persistence
        .register(50, Box::new(GenericPersistence::new(ScriptDataPayload)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_store_survives_a_save_and_a_load() {
        let mut w = World::new();
        let mut m = Map::new();
        m.insert("respect:1".into(), serde_json::json!({ "hale": 130 }));
        m.insert("ins:2".into(), serde_json::json!(7));
        w.resources.insert(ScriptStore(m.clone()));
        let mut out = muo_io::SaveWriter::new(true);
        ScriptDataPayload.serialize(&w, &mut out);
        let bytes = out.into_vec();

        let mut back = World::new();
        let mut r = muo_io::SaveReader::new(&bytes);
        ScriptDataPayload.deserialize(&mut back, &mut r).unwrap();
        assert_eq!(back.resources.get::<ScriptStore>().unwrap().0, m);
    }
}
