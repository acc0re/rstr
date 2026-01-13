use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    CompletedFrame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem},
};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Stdout};
use std::path::PathBuf;
#[derive(Parser)]
#[command(name = "rstr")]
#[command(author = "Alexander Chabowski <alex.gl.cpp@gmail.com>")]
#[command(version = "2026.1.0")]
#[command(about = "A simple search tool with regex support and TUI display", long_about = None)]
struct Cli {
    #[arg(help = "The path in which to search")]
    path: PathBuf,
    #[arg(help = "The search pattern (Regex)")]
    pattern: String,
}

use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let regex = Regex::new(&args.pattern)?;

    let mut terminal = setup_terminal()?;
    
    let mut results = Vec::new();
    let mut last_tick = std::time::Instant::now();
    let mut animation_frame = 0;
    
    let walker = WalkDir::new(&args.path).into_iter().filter_map(|e| e.ok());
    
    for entry in walker {
        let path = entry.path();
        if path.is_file() {
            if last_tick.elapsed() >= std::time::Duration::from_millis(50) {
                animation_frame = (animation_frame + 1) % 4;
                draw_loading(&mut terminal, &args.pattern, path.to_str().unwrap_or(""), animation_frame)?;
                last_tick = std::time::Instant::now();

                //exit during loading
                if event::poll(std::time::Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                            restore_terminal(&mut terminal)?;
                            return Ok(());
                        }
                    }
                }
            }

            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                for (i, line) in reader.lines().enumerate() {
                    if let Ok(line) = line {
                        if regex.is_match(&line) {
                            results.push(format!("{}:{} : {}", path.display(), i + 1, line));
                        }
                    }
                }
            }
        }
    }

    run_ui(&mut terminal, &args.pattern, results)?;
    restore_terminal(&mut terminal)?;

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw_loading(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    pattern: &str,
    current_file: &str,
    animation_frame: usize,
) -> io::Result<()> {
    let dots = match animation_frame {
        1 => ".  ",
        2 => ".. ",
        3 => "...",
        _ => "   ",
    };

    terminal
        .draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(frame.area());

            let header = Block::default()
                .borders(Borders::ALL)
                .title(format!(" Search term: '{}' (Exit: q) ", pattern));

            let loading_text = format!(" Searching{} ", dots);
            let loading_content = format!("Current file: {}", current_file);

            let loading = List::new(vec![ListItem::new(loading_content)])
                .block(Block::default().borders(Borders::ALL).title(loading_text));

            frame.render_widget(header, chunks[0]);
            frame.render_widget(loading, chunks[1]);
        })
        .map(|_| ())
}

fn draw_results<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    pattern: &str,
    results: &[String],
) -> std::io::Result<CompletedFrame<'a>> {
    terminal.draw(|frame| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.area());

        let header = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Search term: '{}' (Exit: q) ", pattern));

        let items: Vec<ListItem> = results.iter().map(|r| ListItem::new(r.as_str())).collect();

        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title(" Found in "));

        frame.render_widget(header, chunks[0]);
        frame.render_widget(list, chunks[1]);
    })
}

fn run_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    pattern: &str,
    results: Vec<String>,
) -> io::Result<()> {
    loop {
        draw_results(terminal, pattern, &results)?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }
    Ok(())
}
