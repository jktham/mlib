use std::{cmp::{max, min}, fs::{self, File}, io::{stdout, Write}, process::Command, time::Duration};
use crossterm::{cursor, event::{self, Event, KeyCode, KeyEvent}, style::{self, Color, Stylize}, terminal, ExecutableCommand, QueueableCommand};

type Err = Box<dyn std::error::Error>;

#[derive(Clone)]
struct Entry {
    path: String,
    name: String,
    is_file: bool,
}

struct State {
    selected: i32,
    path: String,
    entries: Vec<Entry>,
    show_hidden: bool,
    show_help: bool,
}

fn main() -> Result<(), Err> {
    let mut state = State {
        selected: 0,
        path: "/mnt/".to_string(),
        entries: Vec::new(),
        show_hidden: false,
        show_help: false,
    };

    terminal::enable_raw_mode()?;
    stdout().execute(cursor::Hide)?;
    stdout().execute(terminal::EnterAlternateScreen)?;
    
    update(&mut state)?;
    draw(&state)?;

    loop {
        if event::poll(Duration::from_millis(1000))? {
            match event::read()? {
                Event::Key(event) => input(event, &mut state)?,
                Event::FocusGained => (),
                Event::FocusLost => (),
                Event::Mouse(_) => (),
                Event::Paste(_) => (),
                Event::Resize(_, _) => (),
            };

            update(&mut state)?;
            draw(&state)?;
        }
    }
}

fn input(event: KeyEvent, state: &mut State) -> Result<(), Err> {
    match event.code {
        KeyCode::Char('q') => quit()?,
        KeyCode::Up => state.selected -= 1,
        KeyCode::Down => state.selected += 1,
        KeyCode::Right => {
            if state.entries.len() > 0 && !state.entries[state.selected as usize].is_file {
                let p = state.entries[state.selected as usize].path.to_string();
                if !fs::read_dir(p.as_str()).is_err() {
                    state.path = p;
                    state.selected = 0;
                }
            }
        },
        KeyCode::Left => {
            state.path = state.path[0..state.path.rfind("/").unwrap_or(0)].to_string();
            if state.path.len() > 0 {
                state.path = state.path[0..state.path.rfind("/").unwrap_or(0)+1].to_string();
                state.selected = 0;
            } else {
                state.path = "/".to_string();
            }
        },
        KeyCode::Enter => {
            if state.entries.len() > 0 && state.entries[state.selected as usize].is_file {
                let mut p = state.entries[state.selected as usize].path.clone();
                p.pop();
                Command::new("mpv")
                    .arg(p)
                    .stdout(File::create("./log")?)
                    .spawn()?;
            }
        },
        KeyCode::Char('f') => state.show_hidden = !state.show_hidden,
        KeyCode::Char('h') => state.show_help = !state.show_help,
        _ => (),
    };

    return Ok(());
}

fn update(state: &mut State) -> Result<(), Err> {
    let dir = fs::read_dir(state.path.as_str());
    if dir.is_err() {
        return Ok(());
    }
    state.entries.clear();
    for d in dir.unwrap() {
        let e = Entry {
            path: d.as_ref().unwrap().path().as_os_str().to_str().unwrap().to_string() + "/",
            name: d.as_ref().unwrap().file_name().into_string().unwrap(),
            is_file: d.as_ref().unwrap().file_type().unwrap().is_file(),
        };
        if state.show_hidden || !e.name.starts_with(".") && e.name != "System Volume Information" {
            state.entries.push(e);
        }
    }
    state.entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    if state.selected < 0 || state.selected >= state.entries.len() as i32 {
        if state.entries.len() > 0 {
            state.selected = state.selected.rem_euclid(state.entries.len() as i32);
        }
    }

    return Ok(());
}

fn draw(state: &State) -> Result<(), Err> {
    let min_size: (u16, u16) = (10, 3);
    let max_size: (u16, u16) = (400, 20);
    let mut size: (u16, u16) = crossterm::terminal::size()?;
    size = (max(min(size.0, max_size.0), min_size.0), max(min(size.1, max_size.1), min_size.1));

    clear()?;
    draw_rect(0, 0, size.0-1, size.1-1, Color::Red)?;

    draw_text(1, 0, format!(" {0} ", state.path).as_str(), Color::Cyan)?;
    draw_text(size.0-9, size.1-1, " [h]elp ", Color::Cyan)?;

    let mut offset: i32 = 1;
    if state.selected >= size.1 as i32 - 2 {
        offset = size.1 as i32 - state.selected - 2;
    }

    let mut i: i32 = 0;
    for entry in state.entries.clone() {
        let y = i + offset;
        if y > 0 && y < size.1 as i32 - 1 {
            if state.selected == i as i32 {
                draw_text(2, y as u16, ">", Color::White)?;
            }
            if entry.is_file {
                draw_text(4, y as u16, &entry.name, Color::White)?;
            } else {
                draw_text(4, y as u16, &((&entry.name).to_string() + "/"), Color::Cyan)?;
            }
        }
        i += 1;
    }

    if state.show_help {
        draw_fill(size.0-41, 1, size.0-3, size.1-2, ' ', Color::Cyan)?;
        draw_rect(size.0-41, 1, size.0-3, size.1-2, Color::Cyan)?;
        draw_text(size.0-40, 1, " help ", Color::Cyan)?;

        draw_text(size.0-39, min(2, size.1-3), "[arrows]                 navigation", Color::Cyan)?;
        draw_text(size.0-39, min(3, size.1-3), "[enter]                   play file", Color::Cyan)?;
        draw_text(size.0-39, min(4, size.1-3), "[q]                            quit", Color::Cyan)?;
        draw_text(size.0-39, min(5, size.1-3), "[f]             toggle hidden files", Color::Cyan)?;
        draw_text(size.0-39, min(6, size.1-3), "[h]                     toggle help", Color::Cyan)?;
        draw_text(size.0-39, size.1-3,         "[♥]                     mlib v0.1.0", Color::Cyan)?;
    }

    stdout().flush()?;

    return Ok(());
}

fn clear() -> Result<(), Err> {
    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
    return Ok(());
}

fn draw_text(x: u16, y: u16, t: &str, color: Color) -> Result<(), Err> {
    stdout()
        .queue(cursor::MoveTo(x, y))?
        .queue(style::PrintStyledContent(t.with(color)))?
    ;
    return Ok(());
}

fn draw_rect(x1: u16, y1: u16, x2: u16, y2: u16, color: Color) -> Result<(), Err> {
    for x in x1..=x2 {
        for y in y1..=y2 {
            let c: &str = match (x, y) {
                (x, y) if (x, y) == (x1, y1) => "┌",
                (x, y) if (x, y) == (x1, y2) => "└",
                (x, y) if (x, y) == (x2, y1) => "┐",
                (x, y) if (x, y) == (x2, y2) => "┘",
                (x, _) if x == x1 || x == x2 => "│",
                (_, y) if y == y1 || y == y2 => "─",
                (_, _) => "",
            };

            if c != "" {
                stdout()
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent(c.with(color)))?
                ;
            }

        }
    }
    return Ok(());
}

fn draw_fill(x1: u16, y1: u16, x2: u16, y2: u16, c: char, color: Color) -> Result<(), Err> {
    for x in x1..=x2 {
        for y in y1..=y2 {
            stdout()
                .queue(cursor::MoveTo(x, y))?
                .queue(style::PrintStyledContent(c.with(color)))?
            ;
        }
    }
    return Ok(());
}

fn quit() -> Result<(), Err> {
    stdout().execute(terminal::LeaveAlternateScreen)?;
    stdout().execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    std::process::exit(0);
}
