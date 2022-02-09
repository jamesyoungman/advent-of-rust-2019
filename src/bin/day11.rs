use cpu::Processor;
use cpu::Word;
use cpu::{read_program_from_stdin, CpuFault, InputOutputError};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
struct Panel {
    x: i32,
    y: i32,
}

impl Panel {
    fn up(&self) -> Panel {
        Panel {
            y: self.y - 1,
            ..*self
        }
    }
    fn down(&self) -> Panel {
        Panel {
            y: self.y + 1,
            ..*self
        }
    }
    fn right(&self) -> Panel {
        Panel {
            x: self.x + 1,
            ..*self
        }
    }
    fn left(&self) -> Panel {
        Panel {
            x: self.x - 1,
            ..*self
        }
    }
}

impl Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
enum PaintColour {
    White,
    Black,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
enum PaintStatus {
    Unpainted,
    PaintedWhite,
    PaintedBlack,
}

#[derive(Debug)]
struct ShipSurface {
    panels: HashMap<Panel, PaintStatus>,
    total_painted_panels: usize,
}

impl ShipSurface {
    fn new() -> ShipSurface {
        ShipSurface {
            panels: HashMap::new(),
            total_painted_panels: 0,
        }
    }

    fn get_painted_panel_count(&self) -> usize {
        self.total_painted_panels
    }

    fn paint_panel(&mut self, location: Panel, colour: PaintColour) {
        let new_state: PaintStatus = match colour {
            PaintColour::White => PaintStatus::PaintedWhite,
            PaintColour::Black => PaintStatus::PaintedBlack,
        };
        if let Some(PaintStatus::Unpainted) | None = self.panels.insert(location, new_state) {
            self.total_painted_panels += 1;
        }
    }

    fn get_panel_colour(&self, location: &Panel) -> PaintColour {
        match self.panels.get(location) {
            Some(PaintStatus::PaintedWhite) => PaintColour::White,
            _ => PaintColour::Black,
        }
    }
}

impl Display for ShipSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_x = self.panels.keys().map(|p| p.x).max();
        let max_y = self.panels.keys().map(|p| p.y).max();
        let (max_x, max_y) = match (max_x, max_y) {
            (None, _) | (_, None) => {
                // Nothing to display: empty!
                return Ok(());
            }
            (Some(max_x), Some(max_y)) => (max_x, max_y),
        };

        for y in 0..=max_y {
            for x in 0..=max_x {
                let colour = self.get_panel_colour(&Panel { x, y });
                write!(
                    f,
                    "{}",
                    match colour {
                        PaintColour::Black => ' ',
                        PaintColour::White => '#',
                    }
                )?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
enum Heading {
    Up,
    Right,
    Down,
    Left,
}

fn perform_turn_and_move(
    w: Word,
    heading: &mut Heading,
    location: &mut Panel,
) -> Result<(), InputOutputError> {
    let right: bool = w.0 != 0;
    use Heading::*;
    match heading {
        Heading::Up => {
            *heading = if right { Right } else { Left };
            *location = if right {
                location.right()
            } else {
                location.left()
            };
        }
        Heading::Right => {
            *heading = if right { Down } else { Up };
            *location = if right {
                location.down()
            } else {
                location.up()
            };
        }
        Heading::Down => {
            *heading = if right { Left } else { Right };
            *location = if right {
                location.left()
            } else {
                location.right()
            };
        }
        Heading::Left => {
            *heading = if right { Up } else { Down };
            *location = if right {
                location.up()
            } else {
                location.down()
            };
        }
    }
    Ok(())
}

fn run_robot(
    start: Panel,
    start_colour: PaintColour,
    surface: &mut ShipSurface,
    program: &[Word],
) -> Result<Panel, CpuFault> {
    let panel_colour = Arc::new(Mutex::new(start_colour));

    let mut get_input = || -> Result<Word, InputOutputError> {
        match *panel_colour.lock().unwrap() {
            PaintColour::Black => Ok(Word(0)),
            PaintColour::White => Ok(Word(1)),
        }
    };

    let mut moving: bool = false;
    let mut location: Panel = start;
    let mut heading = Heading::Up;

    let mut do_output = |w: Word| -> Result<(), InputOutputError> {
        let new_colour = if moving {
            perform_turn_and_move(w, &mut heading, &mut location)?;
            surface.get_panel_colour(&location)
        } else {
            let new_colour: PaintColour = match w {
                Word(0) => PaintColour::Black,
                Word(1) => PaintColour::White,
                _ => {
                    // invalid; ignore it.
                    return Ok(());
                }
            };
            surface.paint_panel(location.clone(), new_colour);
            new_colour
        };
        moving = !moving;
        *panel_colour.lock().unwrap() = new_colour;
        Ok(())
    };

    let mut cpu: Processor = Processor::new(Word(0));
    cpu.load(Word(0), program)?;
    cpu.run_with_io(&mut get_input, &mut do_output)?;
    Ok(location)
}

fn part1(program: &[Word]) {
    let start = Panel { x: 0, y: 0 };
    let mut surface = ShipSurface::new();
    if let Err(e) = run_robot(start, PaintColour::Black, &mut surface, program) {
        eprintln!("Painting robot crashed: {:?}", e);
    } else {
        println!(
            "Day 11 part 1: panels painted: {}",
            surface.get_painted_panel_count()
        );
    }
}

fn part2(program: &[Word]) {
    let start = Panel { x: 0, y: 0 };
    let mut surface = ShipSurface::new();
    if let Err(e) = run_robot(start, PaintColour::White, &mut surface, program) {
        eprintln!("Painting robot crashed: {:?}", e);
    } else {
        println!("Day 11 part 2\n{}", surface);
    }
}

fn main() {
    match read_program_from_stdin() {
        Ok(words) => {
            part1(&words);
            part2(&words);
        }
        Err(e) => {
            eprintln!("failed to load program: {}", e);
        }
    }
}
