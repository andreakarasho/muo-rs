//! The Khaldun books (GrimmochJournal, LysanderNotebook, TavarasJournal): `base(Utility.Random(0xFF1, 2), false)`
//! with a fixed `BookContent`, both in the class template (`Data/Templates/Items/books/khaldun.json`).

use muo_items::books::{BASE_BOOK, new_book};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

/// `new <Book>()`: the graphic and the text are the class template's.
pub fn khaldun_book_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_book(w, ty, 0, None, None, 0, false);
    muo_content::templates::apply_item_as(w, id, ty.full_name);
    id
}

macro_rules! khaldun_books {
    ($($name:ident = $full:literal;)*) => {
        $(
            #[doc = concat!("`", $full, "` `[SerializationGenerator(0)]`.")]
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&BASE_BOOK),
                construct: Some(|w| khaldun_book_new(w, &$name)),
                serialize: |w, id, out| {
                    (BASE_BOOK.serialize)(w, id, out);
                    out.write_encoded_int(0);
                },
                deserialize: |w, id, r| {
                    (BASE_BOOK.deserialize)(w, id, r)?;
                    r.read_encoded_int()?;
                    Ok(())
                },
                ..BASE_BOOK
            };
        )*

        /// Every class of this file.
        pub static ALL: &[&ItemType] = &[$(&$name,)*];
    };
}

khaldun_books! {
    GRIMMOCH_JOURNAL1 = "Server.Items.GrimmochJournal1";
    GRIMMOCH_JOURNAL2 = "Server.Items.GrimmochJournal2";
    GRIMMOCH_JOURNAL3 = "Server.Items.GrimmochJournal3";
    GRIMMOCH_JOURNAL6 = "Server.Items.GrimmochJournal6";
    GRIMMOCH_JOURNAL7 = "Server.Items.GrimmochJournal7";
    GRIMMOCH_JOURNAL11 = "Server.Items.GrimmochJournal11";
    GRIMMOCH_JOURNAL14 = "Server.Items.GrimmochJournal14";
    GRIMMOCH_JOURNAL17 = "Server.Items.GrimmochJournal17";
    GRIMMOCH_JOURNAL23 = "Server.Items.GrimmochJournal23";
    LYSANDER_NOTEBOOK1 = "Server.Items.LysanderNotebook1";
    LYSANDER_NOTEBOOK2 = "Server.Items.LysanderNotebook2";
    LYSANDER_NOTEBOOK3 = "Server.Items.LysanderNotebook3";
    LYSANDER_NOTEBOOK7 = "Server.Items.LysanderNotebook7";
    LYSANDER_NOTEBOOK8 = "Server.Items.LysanderNotebook8";
    LYSANDER_NOTEBOOK11 = "Server.Items.LysanderNotebook11";
    TAVARAS_JOURNAL1 = "Server.Items.TavarasJournal1";
    TAVARAS_JOURNAL2 = "Server.Items.TavarasJournal2";
    TAVARAS_JOURNAL3 = "Server.Items.TavarasJournal3";
    TAVARAS_JOURNAL6 = "Server.Items.TavarasJournal6";
    TAVARAS_JOURNAL7 = "Server.Items.TavarasJournal7";
    TAVARAS_JOURNAL8 = "Server.Items.TavarasJournal8";
    TAVARAS_JOURNAL9 = "Server.Items.TavarasJournal9";
    TAVARAS_JOURNAL11 = "Server.Items.TavarasJournal11";
    TAVARAS_JOURNAL14 = "Server.Items.TavarasJournal14";
    TAVARAS_JOURNAL16 = "Server.Items.TavarasJournal16";
    TAVARAS_JOURNAL16B = "Server.Items.TavarasJournal16b";
    TAVARAS_JOURNAL17 = "Server.Items.TavarasJournal17";
    TAVARAS_JOURNAL19 = "Server.Items.TavarasJournal19";
}

pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
}
