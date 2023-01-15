use std::{str::FromStr, time::Duration};

use indent::indent_by;
use termix::{
    color::{Color, StyledText},
    event::Event,
    key::Key,
    model::{ModelAct, Updater},
    Program,
};

const PROGRESS_BAR_WIDTH: u64 = 71;

const PROGRESS_FULL_CHAR: &str = "█";
const PROGRESS_EMPTY_CHAR: char = '░';
fn ramp() -> Vec<Color> {
    make_ramp()
}

// I should write color blending, but I'm lazy...
fn make_ramp() -> Vec<Color> {
    let color_list = vec![
        "#b14ffe", "#af53fd", "#ae58fb", "#ac5cfa", "#ab60f8", "#a964f7", "#a868f6", "#a76cf4",
        "#a66ff3", "#a472f2", "#a375f1", "#a279ef", "#a07cee", "#9f7eed", "#9e81ec", "#9d84eb",
        "#9b87ea", "#9a8ae9", "#998ce7", "#988fe6", "#9791e5", "#9594e4", "#9496e3", "#9399e2",
        "#929be1", "#909ee0", "#8fa0df", "#8ea2de", "#8ca5dd", "#8ba7dc", "#8aa9db", "#88acda",
        "#87aed9", "#85b0d7", "#84b2d6", "#83b4d5", "#81b7d4", "#80b9d3", "#7ebbd2", "#7cbdd1",
        "#7bbfd0", "#79c1cf", "#77c4cd", "#76c6cc", "#74c8cb", "#72caca", "#70ccc9", "#6ecec7",
        "#6cd0c6", "#6ad2c5", "#68d4c4", "#66d6c2", "#64d8c1", "#61dac0", "#5fdcbe", "#5cdebd",
        "#5ae0bc", "#57e2ba", "#54e5b9", "#51e7b7", "#4ee9b6", "#4aebb4", "#47edb2", "#43efb1",
        "#3ff1af", "#3af3ae", "#35f5ac", "#2ff7aa", "#28f9a8", "#20fba6", "#14fda4",
    ]
    .iter()
    .map(|hex| Color::from_str(hex).unwrap())
    .collect();
    color_list
}

struct Model {
    choice: i8,
    chosen: bool,
    ticks: i16,
    quitting: bool,
    frames: usize,
    loaded: bool,
    progress: f64,
}

impl Model {
    fn new() -> Model {
        Model {
            chosen: false,
            choice: 0,
            ticks: 10,
            quitting: false,
            frames: 0,
            loaded: false,
            progress: 0f64,
        }
    }
}

#[derive(Debug)]
enum CustomEvent {
    Tick,
    Frame,
}

impl ModelAct<Model, CustomEvent> for Model {
    fn update(&self, event: &Event<CustomEvent>) -> Updater<Model, CustomEvent> {
        if !self.chosen {
            return update_choices(event, self);
        }
        return update_chosen(event, self);
    }
    fn view(&self) -> String {
        if self.quitting {
            return String::from("\n  See you later!\n\n");
        }
        let s = if !self.chosen {
            choices_view(self)
        } else {
            chosen_view(self)
        };
        indent_by(2, format!("\n{s}\n\n"))
    }
}

fn update_choices(event: &Event<CustomEvent>, model: &Model) -> Updater<Model, CustomEvent> {
    match event {
        Event::Init => (Some(Box::new(Model { ..*model })), Some(tick)),
        Event::Keyboard(Key::Down | Key::Char('j')) => {
            let mut choice = model.choice + 1;
            if model.choice + 1 > 3 {
                choice = 3;
            }
            (Some(Box::new(Model { choice, ..*model })), None)
        }
        Event::Keyboard(Key::Up | Key::Char('k')) => {
            let mut choice = model.choice - 1;
            if model.choice - 1 < 0 {
                choice = 0;
            }
            (Some(Box::new(Model { choice, ..*model })), None)
        }
        Event::Keyboard(Key::Enter) => (
            Some(Box::new(Model {
                chosen: true,
                ..*model
            })),
            Some(frame),
        ),
        Event::Keyboard(Key::ESC | Key::Char('q') | Key::Ctrl('c')) => (
            Some(Box::new(Model {
                quitting: true,
                ..*model
            })),
            Some(|| Event::Quit),
        ),
        Event::Custom(CustomEvent::Tick) => {
            if model.ticks == 0 {
                return (
                    Some(Box::new(Model {
                        quitting: true,
                        ..*model
                    })),
                    Some(|| Event::Quit),
                );
            }
            (
                Some(Box::new(Model {
                    ticks: model.ticks - 1,
                    ..*model
                })),
                Some(tick),
            )
        }
        _ => (None, None),
    }
}

fn update_chosen(event: &Event<CustomEvent>, model: &Model) -> Updater<Model, CustomEvent> {
    match event {
        Event::Custom(CustomEvent::Frame) => {
            if !model.loaded {
                let frames = model.frames + 1;
                let progress = outbounce((frames) as f64 / 100f64);
                if progress >= 1f64 {
                    let progress = 1f64;
                    let loaded = true;
                    let ticks = 10;
                    return (
                        Some(Box::new(Model {
                            progress,
                            loaded,
                            ticks,
                            frames,
                            ..*model
                        })),
                        Some(tick),
                    );
                }
                return (
                    Some(Box::new(Model {
                        frames,
                        progress,
                        ..*model
                    })),
                    Some(frame),
                );
            }
            (Some(Box::new(Model { ..*model })), None)
        }
        Event::Custom(CustomEvent::Tick) => {
            if model.loaded {
                if model.ticks == 0 {
                    return (
                        Some(Box::new(Model {
                            quitting: true,
                            ..*model
                        })),
                        Some(|| Event::Quit),
                    );
                }
                return (
                    Some(Box::new(Model {
                        ticks: model.ticks - 1,
                        ..*model
                    })),
                    Some(tick),
                );
            }
            (Some(Box::new(Model { ..*model })), None)
        }
        Event::Keyboard(Key::ESC | Key::Char('q') | Key::Ctrl('c')) => (
            Some(Box::new(Model {
                quitting: true,
                ..*model
            })),
            Some(|| Event::Quit),
        ),
        _ => (None, None),
    }
}

fn tick() -> Event<CustomEvent> {
    let one_sec = Duration::from_secs(1);
    std::thread::sleep(one_sec);
    Event::Custom(CustomEvent::Tick)
}

fn frame() -> Event<CustomEvent> {
    let one_frame = Duration::from_secs_f64(1f64 / 60f64);
    std::thread::sleep(one_frame);
    Event::Custom(CustomEvent::Frame)
}

fn main() {
    Program::new(Box::new(Model::new())).run();
}

fn outbounce(t: f64) -> f64 {
    if t < (4f64 / 11.0) {
        (121f64 * t * t) / 16.0
    } else if t < (8f64 / 11.0) {
        return (363f64 / 40.0 * t * t) - (99f64 / 10.0 * t) + (17f64 / 5.0);
    } else if t < (9f64 / 10.0) {
        return (4356f64 / 361.0 * t * t) - (35442f64 / 1805.0 * t) + (16061f64 / 1805.0);
    } else {
        return (54f64 / 5.0 * t * t) - (513f64 / 25.0 * t) + (268f64 / 25.0);
    }
}
fn checkbox(label: &str, checked: bool) -> String {
    if checked {
        return StyledText::new(
            &format!("[x] {label}"),
            Some(Color::Ansi256(212)),
            None,
            None,
            None,
            None,
        )
        .text();
    }
    format!(r#"[ ] {label}"#)
}

fn subtle(text: &str) -> String {
    StyledText::new(text, Some(Color::Ansi256(241)), None, None, None, None).text()
}
fn keyword(text: &str) -> String {
    StyledText::new(text, Some(Color::Ansi256(241)), None, None, None, None).text()
}

fn dot() -> String {
    StyledText::new(" ・ ", Some(Color::Ansi256(241)), None, None, None, None).text()
}

fn choices_view(model: &Model) -> String {
    let c = model.choice;
    let mut tpl = String::new();

    tpl += "What to do today?\n\n";
    let choices = format!(
        "{}\n{}\n{}\n{}",
        checkbox("Plant carrots", c == 0),
        checkbox("Go to the market", c == 1),
        checkbox("Read something", c == 2),
        checkbox("See friends", c == 3)
    );
    tpl += &format!("{}\n\n", choices);
    tpl += &format!(
        "Program quits in {} seconds\n\n",
        StyledText::new(
            &model.ticks.to_string(),
            Some(Color::Ansi256(79)),
            None,
            None,
            None,
            None
        )
        .text()
    );
    tpl += &(subtle("j/k, up/down: select")
        + &dot()
        + &subtle("enter: choose")
        + &dot()
        + &subtle("q, esc: quit"));

    tpl
}
fn progressbar(percent: f64) -> String {
    let w = PROGRESS_BAR_WIDTH as f64;
    let full_size = (w * percent).round() as usize;
    let mut full_cells = String::new();
    let ramp_res = ramp();
    (0..full_size).for_each(|i| {
        full_cells += &StyledText::new(
            PROGRESS_FULL_CHAR,
            Some(ramp_res[i].clone()),
            None,
            None,
            None,
            None,
        )
        .text();
    });

    let empty_size = w as usize - full_size;
    // let empty_cells = w as usize + full_size;
    let empty_cells: String = (0..empty_size).map(|_| PROGRESS_EMPTY_CHAR).collect();

    format!(
        "{}{} {:3.0}",
        full_cells,
        empty_cells,
        (percent * 100f64).round(),
    )
}
// The second view, after a task has been chosen
fn chosen_view(m: &Model) -> String {
    let msg = match m.choice {
        0 => format!(
            "Carrot planting?\n\nCool, we'll need {} and {}...",
            keyword("libgarden"),
            keyword("vegeutils")
        ),
        1 => format!(
            "A trip to the market?\n\nOkay, then we should install {} and {}...",
            keyword("marketkit"),
            keyword("libshopping")
        ),
        2 => format!(
            "Reading time?\n\nOkay, cool, then we’ll need a library. Yes, an {}.",
            keyword("actual library")
        ),
        _ => format!(
            "It’s always good to see friends.\n\nFetching {} and {}...",
            keyword("social-skills"),
            keyword("conversationutils")
        ),
    };

    let label = if m.loaded {
        format!(
            "Downloaded. Exiting in {} seconds...",
            StyledText::new(
                &m.ticks.to_string(),
                Some(Color::Ansi256(79)),
                None,
                None,
                None,
                None
            )
            .text()
        )
    } else {
        String::from("Downloading...")
    };

    msg + "\n\n" + &label + "\n" + &progressbar(m.progress) + "%"
}
