//! UI Code for TMIX
mod ui;

use std::{
    io::{self, Result},
    thread,
    time::{Duration, SystemTime},
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
use tmix::pulse_api::{PulseAPI, VolumeInfo};

const APP_NAME: &str = "TMIX";

/// Application Manager For TMIX
pub struct App {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    api: PulseAPI,
}

impl App {
    pub fn try_new() -> Result<Self> {
        let mut api = PulseAPI::new();
        api.startup_connection()?;
        Ok(Self {
            terminal: None,
            api,
        })
    }

    /// Launch Terminal Process and begin Listening for events
    pub fn run(&mut self) -> Result<()> {
        // Setup a Main Loop
        self.start_up_tui()?;
        // Do Some Thing!
        let now = SystemTime::now();
        while now.elapsed().expect("Something broke in time") < Duration::from_millis(15000) {
            let data = self.api.get_volume_info()?;
            self.draw_data(data)?;
            thread::sleep(Duration::from_millis(10));
        }
        self.shut_down_api();
        self.shut_down_tui()
    }

    fn start_up_tui(&mut self) -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        self.terminal = Some(Terminal::new(backend)?);

        Ok(())
    }

    fn draw_data(&mut self, data: VolumeInfo) -> Result<()> {
        self.terminal
            .as_mut()
            .expect("don't draw till intialized")
            .draw(|f| {
                let size = f.size();
                let block = Block::default().title(APP_NAME).borders(Borders::ALL);
                f.render_widget(block, size);
                ui(f, data);
            })?;
        Ok(())
    }

    fn shut_down_tui(&mut self) -> Result<()> {
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

    fn shut_down_api(&mut self) {
        self.api.shutdown();
    }
}

