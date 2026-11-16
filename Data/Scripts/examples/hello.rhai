// `[hello [name]`: a command defined by a script. The `///` header above a function makes it a command:
// `@command <Name>` names it, `@access <level>` sets who may use it (Player, Counselor, GameMaster, Seer,
// Administrator, Developer, Owner; GameMaster when absent). `args` is the text after the command.

/// @command Hello
/// @access GameMaster
fn hello(m, args) {
    if args == "" {
        say(m, "Hello!");
    } else {
        say(m, `Hello, ${args}!`);
    }
    log(`${m.name} said hello`);
}

// `@event <name>` runs a function on a server event. Uncomment to greet every player on login:
//
// /// @event login
// fn welcome(m) {
//     send_message(m, `Welcome, ${m.name}.`);
// }
//
// The others are `speech` (`fn x(m, text)`, return true to mark the speech handled) and `server_started` (`fn x()`).
