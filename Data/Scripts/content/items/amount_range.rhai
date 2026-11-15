// The `(amountFrom, amountTo)` constructor of a resource: `Amount = Utility.RandomMinMax(amountFrom, amountTo)`.
// The plain constructor passes no range and keeps the template's amount.
fn on_spawn(e, from, to) {
    if type_of(from) == "i64" && type_of(to) == "i64" {
        e.amount = random(from, to);
    }
}
