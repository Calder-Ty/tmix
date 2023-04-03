//! UI Code for TMIX

use std::{
    io::{self, Result},
    thread,
    time::Duration,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

use self::ui::ui;

const APP_NAME: &str = "TMIX";

/// Application Manager For TMIX
#[derive(Default)]
pub struct App {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
}

impl App {
    /// Launch Terminal Process and begin Listening for events
    pub fn run(&mut self) -> Result<()> {
        self.start_up()?;
        // Do Some Thing!
        thread::sleep(Duration::from_millis(5000));
        self.shut_down()
    }

    fn start_up(&mut self) -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        self.terminal = Some(Terminal::new(backend)?);

        self.terminal
            .as_mut()
            .expect("Just Initailized")
            .draw(|f| {
                let size = f.size();
                let block = Block::default().title(APP_NAME).borders(Borders::ALL);
                f.render_widget(block, size);
                ui(f);
            })?;
        Ok(())
    }

    fn shut_down(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal
                .as_mut()
                .expect("Don't Shut Down without Initiating")
                .backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal
            .as_mut()
            .expect("Don't Shut Down without Initiating")
            .show_cursor()?;
        Ok(())
    }
}

/// Module for defining UI stuff
mod ui {

    use tui::{
        backend::Backend,
        layout::{Constraint, Direction, Layout},
        style::{Color, Style},
        widgets::{Block, Borders, Gauge, BarChart},
        Frame,
    };

    pub(crate) fn ui<B: Backend>(f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(f.size());
        for (i, chunk) in chunks.into_iter().enumerate() {
            let data = &[("volume", 20 * i as u64)];
            let block = Block::default()
                .title(format!("Window #{}", i))
                .borders(Borders::ALL);
            let bar = BarChart::default()
                .block(block)
                .bar_style(Style::default().fg(Color::DarkGray))
                .data(data)
                .max(100);
            f.render_widget(bar, chunk);
        }
    }
}
