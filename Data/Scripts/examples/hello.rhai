// `[hello [name]`: a command defined by a script. `fn command_<name>(m, args)` registers `[<name>`; the
// `// access:` line right above it sets who may use it (Player, Counselor, GameMaster, Seer, Administrator,
// Developer, Owner; GameMaster when absent). `args` is the text after the command.

// access: GameMaster
fn command_hello(m, args) {
    if args == "" {
        say(m, "Hello!");
    } else {
        say(m, `Hello, ${args}!`);
    }
    log(`${m.name} said hello`);
}

// Any script can also define the global hooks. Uncomment to greet every player on login:
//
// fn on_login(m) {
//     send_message(m, `Welcome, ${m.name}.`);
// }
//
// The others are `on_speech(m, text)` (return true to mark the speech handled) and `on_server_started()`.
