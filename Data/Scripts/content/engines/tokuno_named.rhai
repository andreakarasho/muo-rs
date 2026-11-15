// Server.Items.AncientUrn and Server.Items.HonorableSwords (Data/Templates/Items/engines/misc.json): the name
// they are made with. `new AncientUrn()` picks one of these names (`Names.RandomElement()`, HonorableSwords uses
// the urn's list); `new AncientUrn(name)` is handed it.

fn names() {
    [
        "Akira", "Avaniaga", "Aya", "Chie", "Emiko", "Fumiyo", "Gennai", "Gennosuke", "Genjo", "Hamato", "Harumi",
        "Ikuyo", "Juri", "Kaori", "Kaoru", "Kiyomori", "Mayako", "Motoki", "Musashi", "Nami", "Nobukazu", "Roku",
        "Romi", "Ryo", "Sanzo", "Sakamae", "Satoshi", "Takamori", "Takuro", "Teruyo", "Toshiro", "Yago", "Yeijiro",
        "Yoshi", "Zeshin"
    ]
}

fn on_spawn(item) {
    on_spawn(item, pick(names()));
}

fn on_spawn(item, name) {
    // the urn's graphic tells the two classes apart
    if item.item_id == 0x241D {
        set_prop(item, "UrnName", name);
    } else {
        set_prop(item, "SwordsName", name);
    }
}
