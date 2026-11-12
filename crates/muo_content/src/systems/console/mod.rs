//! Console commands: the `save`, `shutdown` and `restart` console commands.
//!
//! Console commands run on the reader thread, where there is no `World`. They post their work to the game loop
//! (`Core.LoopContext.Post`), which is also what `Core.Kill` effectively does (it only sets flags and wakes the
//! loop). The binary owns the [`ConsoleInputHandler`], so `Configure` reads it from the [`ConsoleHandle`]
//! resource it sets before the `Configure` phase runs (`muo/src/main.rs`).

use muo_server::World;
use muo_server::console::ConsoleInputHandler;
use muo_server::core_loop::EventLoopContext;

use crate::world_saves::auto_save;

/// The console handler, set by the binary.
#[derive(Default)]
pub struct ConsoleHandle(pub Option<ConsoleInputHandler>);

/// `ConsoleCommands.Configure()`.
pub fn configure(w: &mut World) {
    let Some(handler) = w.resources.get_mut::<ConsoleHandle>().0.clone() else {
        return;
    };
    let ctx = w.core.loop_context.clone();
    register(&handler, &ctx);
}

/// Registers `save|s`, `shutdown|sh` and `restart|r`.
pub fn register(handler: &ConsoleInputHandler, ctx: &EventLoopContext) {
    let c = ctx.clone();
    handler.register_commands(&["save", "s"], "Saves the world", move |_| {
        c.post(auto_save::save)
    });
    let c = ctx.clone();
    handler.register_commands(&["shutdown", "sh"], "Shuts down the server.", move |_| {
        c.post(|w| w.core.kill(false))
    });
    let c = ctx.clone();
    handler.register_commands(&["restart", "r"], "Restarts the server.", move |_| {
        c.post(|w| w.core.kill(true))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn commands_are_registered_under_their_aliases_and_post_to_the_loop() {
        let handler = ConsoleInputHandler::new(true, Arc::new(AtomicBool::new(false)));
        let ctx = EventLoopContext::new(8);
        register(&handler, &ctx);
        for name in ["save", "s", "shutdown", "sh", "restart", "r"] {
            assert!(handler.get_input_command(name).is_some(), "{name}");
        }
        assert!(ctx.is_empty());
        (handler.get_input_command("sh").unwrap())("");
        assert!(!ctx.is_empty());
        let mut w = World::new();
        ctx.execute_tasks(&mut w);
        assert!(ctx.is_empty());
    }
}
