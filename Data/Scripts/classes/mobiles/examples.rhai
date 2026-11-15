mobile("Examples.ShoutingRat", |c| {
    c.extends = "Rat";
    c.script = "examples/shouting_rat.rhai";
    c.on_create(|m| {
        m.name = "a shouting rat";
    });
});
