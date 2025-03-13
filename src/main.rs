use std::{cmp::{max, min}, fs::{self, File}, io::{stdout, Write}, process::Command, time::Duration};
use crossterm::{cursor, event::{self, Event, KeyCode, KeyEvent}, style::{self, Color, Stylize}, terminal, ExecutableCommand, QueueableCommand};
use serde::{Serialize, Deserialize};

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

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    default_path: String,
    player: String,
}

fn main() -> Result<(), Err> {
    let mut config = Config {
        default_path: dirs::home_dir().unwrap().to_str().unwrap_or("/").to_string(),
        player: "mpv".to_string(),
    };

    if fs::exists("./config.json")? {
        let s = fs::read_to_string("./config.json")?;
        config = serde_json::from_str(s.as_str())?;
    } else {
        let s = serde_json::to_string_pretty(&config)?;
        fs::write("./config.json", s)?;
    }

    let mut state = State {
        selected: 0,
        path: config.default_path.clone(),
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
                Event::Key(event) => input(event, &mut state, &config)?,
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

fn input(event: KeyEvent, state: &mut State, config: &Config) -> Result<(), Err> {
    match event.code {
        KeyCode::Char('q') => quit()?,
        KeyCode::Up => state.selected -= 1,
        KeyCode::Down => state.selected += 1,
        KeyCode::Right => {
            if state.entries.len() > 0 && !state.entries[state.selected as usize].is_file {
                let p = state.entries[state.selected as usize].path.to_string();
                if fs::read_dir(p.as_str()).is_ok() {
                    state.path = p;
                    state.selected = 0;
                }
            }
        },
        KeyCode::Left => {
            state.path = state.path[0..state.path.rfind("/").unwrap_or(0)].to_string();
            state.selected = 0;
            if state.path == "" {
                state.path = "/".to_string();
            }
        },
        KeyCode::Enter => {
            if state.entries.len() > 0 && state.entries[state.selected as usize].is_file {
                Command::new(config.player.clone())
                    .arg(state.entries[state.selected as usize].path.clone())
                    .stdout(File::create("./out.log")?)
                    .stderr(File::create("./err.log")?)
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
    for d in dir? {
        let mut e = Entry {
            path: d.as_ref().unwrap().path().as_os_str().to_str().unwrap().to_string(),
            name: d.as_ref().unwrap().file_name().into_string().unwrap(),
            is_file: d.as_ref().unwrap().file_type().unwrap().is_file(),
        };
        let suffix_filter = [".mp4", ".mkv", ".avi", ".m4v", ".webm"];
        if state.show_hidden || !e.name.starts_with(".") && e.name != "System Volume Information" && (!e.is_file || suffix_filter.iter().any(|s| e.name.ends_with(s))) {
            if !state.show_hidden {
                for s in suffix_filter {
                    e.name = e.name.replace(s, "");
                }
            }
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
    let min_size: (i32, i32) = (0, 0);
    let max_size: (i32, i32) = (400, 20);
    let mut size: (i32, i32) = (crossterm::terminal::size()?.0 as i32, crossterm::terminal::size()?.1 as i32);
    size = (max(min(size.0, max_size.0), min_size.0), max(min(size.1, max_size.1), min_size.1));

    clear()?;
    draw_rect(0, 0, size.0-1, size.1-1, Color::Red)?;

    draw_text(1, 0, format!(" {0}{1} ", state.path, if state.path == "/" {""} else {"/"}).as_str(), Color::Cyan)?;
    draw_text(size.0-9, size.1-1, " [h]elp ", Color::Cyan)?;

    let mut offset: i32 = 1;
    if state.selected >= size.1 as i32 - 2 {
        offset = size.1 as i32 - state.selected - 2;
    }

    let mut i: i32 = 0;
    for entry in state.entries.clone() {
        let y = i + offset;
        if y > 0 && y < size.1 - 1 {
            if state.selected == i {
                draw_text(2, y, ">", Color::White)?;
            }
            if entry.is_file {
                draw_text(4, y, &entry.name, Color::White)?;
            } else {
                draw_text(4, y, &((&entry.name).to_string() + "/"), Color::Cyan)?;
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
        draw_text(size.0-39, min(5, size.1-3), "[f]                   toggle filter", Color::Cyan)?;
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

fn draw_text(x: i32, y: i32, t: &str, color: Color) -> Result<(), Err> {
    if x < 0 || y < 0 {
        return Ok(());
    }
    stdout()
        .queue(cursor::MoveTo(x as u16, y as u16))?
        .queue(style::PrintStyledContent(t.with(color)))?
    ;
    return Ok(());
}

fn draw_rect(x1: i32, y1: i32, x2: i32, y2: i32, color: Color) -> Result<(), Err> {
    if x1 < 0 || y1 < 0 || x1 > x2 || y1 > y2 {
        return Ok(());
    }
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
                    .queue(cursor::MoveTo(x as u16, y as u16))?
                    .queue(style::PrintStyledContent(c.with(color)))?
                ;
            }

        }
    }
    return Ok(());
}

fn draw_fill(x1: i32, y1: i32, x2: i32, y2: i32, c: char, color: Color) -> Result<(), Err> {
    if x1 < 0 || y1 < 0 || x1 > x2 || y1 > y2 {
        return Ok(());
    }
    for x in x1..=x2 {
        for y in y1..=y2 {
            stdout()
                .queue(cursor::MoveTo(x as u16, y as u16))?
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
