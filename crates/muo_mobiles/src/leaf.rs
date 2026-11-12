//! `creature_leaf!`: one concrete creature class (a `MobileType` plus its `CreatureType` table of overrides).
//! The save is "parent part + version int 0" (`encoded` for `[SerializationGenerator(0)]` without `false`).

macro_rules! creature_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, ctor $ctor:expr, encoded $enc:literal;
        c { $($cf:ident : $cv:expr,)* }
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        /// The creature-family overrides of the class.
        pub static $class: muo_content::creature::CreatureType =
            muo_content::creature::CreatureType { $($cf: $cv,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
            full_name: $full,
            parent: Some(&$parent),
            construct: $ctor,
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                if $enc {
                    out.write_encoded_int(0);
                } else {
                    out.write_i32(0);
                }
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = if $enc { r.read_encoded_int()? } else { r.read_i32()? };
                Ok(())
            },
            $($mf: $mv,)*
            ..$parent
        };
    };

    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, ctor $ctor:expr, state $ser:path, $de:path;
        c { $($cf:ident : $cv:expr,)* }
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        /// The creature-family overrides of the class.
        pub static $class: muo_content::creature::CreatureType =
            muo_content::creature::CreatureType { $($cf: $cv,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
            full_name: $full,
            parent: Some(&$parent),
            construct: $ctor,
            family: Some(&$class),
            // The class writes its own version and fields after the parent's part.
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $ser(w, id, out);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                $de(w, id, r)
            },
            $($mf: $mv,)*
            ..$parent
        };
    };
}
pub(crate) use creature_leaf;
