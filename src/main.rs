use std::{cmp::min, fs, io::{stdout, Write}, time::Duration};
use crossterm::{cursor, event::{self, Event, KeyCode, KeyEvent}, style::{self, Color, Stylize}, terminal, ExecutableCommand, QueueableCommand};
type Err = Box<dyn std::error::Error>;

fn main() -> Result<(), Err> {
    terminal::enable_raw_mode()?;
    stdout().execute(cursor::Hide)?;
    stdout().execute(terminal::EnterAlternateScreen)?;
    
    draw()?;

    loop {
        if event::poll(Duration::from_millis(1000))? {
            match event::read()? {
                Event::Key(event) => input(event)?,
                Event::FocusGained => (),
                Event::FocusLost => (),
                Event::Mouse(_) => (),
                Event::Paste(_) => (),
                Event::Resize(_, _) => (),
            };

            draw()?;
        }
    }
}

fn input(event: KeyEvent) -> Result<(), Err> {
    match event.code {
        KeyCode::Char('q') => quit()?,
        _ => (),
    };

    return Ok(());
}

fn draw() -> Result<(), Err> {
    let fg: Color = Color::White;
    let bg: Color = Color::White;
    let max_size: (u16, u16) = (120, 20);
    let mut size: (u16, u16) = crossterm::terminal::size()?;
    size = (min(size.0, max_size.0), min(size.1, max_size.1));

    clear()?;
    draw_rect(0, 0, size.0, size.1, bg)?;

    let paths = fs::read_dir("./").unwrap();
    let mut y = 1;
    for path in paths {
        draw_text(2, y, path.unwrap().path().as_os_str().to_str().unwrap(), fg)?;
        y += 1;
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

fn quit() -> Result<(), Err> {
    stdout().execute(terminal::LeaveAlternateScreen)?;
    stdout().execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    std::process::exit(0);
}
