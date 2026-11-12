//! WelcomeTimer: spouts some welcome messages to a user at a set interval. It is
//! used on character creation and login.

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages::send_message_hue;
use muo_server::timer;

use super::server_list;

/// `m_Messages`, built in `Initialize`.
#[derive(Default)]
pub struct WelcomeMessages(pub Vec<String>);

/// `WelcomeTimer.Initialize`.
pub fn initialize(w: &mut World) {
    let name = server_list::server_name(w);
    w.resources.get_mut::<WelcomeMessages>().0 =
        if crate::systems::special_systems::test_center::enabled(w) {
            [
            format!("Welcome to {name}."),
            "You are able to customize your character's stats and skills at anytime to anything you wish.".to_owned(),
            "To see the commands to do this just say 'help'.".to_owned(),
            "You will find a bank check worth 1,000,000 gold in your bank!".to_owned(),
            "A spellbook and a bag of reagents has been placed into your bank box.".to_owned(),
            "Various tools have been placed into your bank.".to_owned(),
            "Various raw materials like ingots, logs, feathers, hides, bottles, etc, have been placed into your bank.".to_owned(),
            "5 unmarked recall runes, 5 Felucca moonstones and 5 Trammel moonstones have been placed into your bank box.".to_owned(),
            "One of each level of treasure map has been placed in your bank box.".to_owned(),
            "You will find 9000 silver pieces deposited into your bank box. Spend it as you see fit and enjoy yourself!".to_owned(),
            "You will find 9000 gold pieces deposited into your bank box. Spend it as you see fit and enjoy yourself!".to_owned(),
            "A bag of PowerScrolls has been placed in your bank box.".to_owned(),
        ]
        .to_vec()
        } else {
            vec![
                format!("Welcome to {name}."),
                "Please enjoy your stay.".to_owned(),
            ]
        };
}

/// `new WelcomeTimer(m).Start()`.
pub fn start(w: &mut World, m: MobileId) {
    let count = w.resources.get_mut::<WelcomeMessages>().0.len();
    start_count(w, m, count);
}

/// `new WelcomeTimer(m, count).Start()`.
pub fn start_count(w: &mut World, m: MobileId, count: usize) {
    let mut state = 0;
    timer::start(
        w,
        TimeSpan::from_secs(5),
        TimeSpan::from_secs(10),
        0,
        move |w| {
            if state < count {
                let text = w
                    .resources
                    .get_mut::<WelcomeMessages>()
                    .0
                    .get(state)
                    .cloned();
                state += 1;
                if let Some(t) = text {
                    send_message_hue(w, m, 0x35, &t);
                }
                return;
            }
            if let Some(h) = timer::current(w) {
                timer::stop(w, h);
            }
        },
    );
}
