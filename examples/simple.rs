use std::time::Duration;

use termix::{
    color::{Color, StyledText},
    event::Event,
    model::{ModelAct, Updater},
    Program,
};

struct Model(usize);

#[derive(Debug)]
struct Tick {}

impl ModelAct<Model, Tick> for Model {
    fn update(&self, event: &Event<Tick>) -> Updater<Model, Tick> {
        match event {
            Event::Init => (Some(Box::new(Model(self.0))), Some(tick)),
            Event::Custom(_) => {
                let next = self.0 - 1;
                if next == 0 {
                    return (Some(Box::new(Model(next))), Some(|| Event::Quit));
                }
                (Some(Box::new(Model(next))), Some(tick))
            }
            Event::Keyboard(..) => (None, Some(|| Event::Quit)),
            _ => (None, None),
        }
    }
    fn view(&self) -> String {
        StyledText::new(
            &format!(
                "Hi. This program will exit in {} seconds. To quit sooner press any key.\n",
                self.0
            ),
            Some(Color::Ansi256(212)),
            None,
            None,
            None,
            None,
        )
        .text()
    }
}

fn tick() -> Event<Tick> {
    let one_sec = Duration::from_secs(1);
    std::thread::sleep(one_sec);
    Event::Custom(Tick {})
}

fn main() {
    Program::new(Box::new(Model(5))).run();
}
