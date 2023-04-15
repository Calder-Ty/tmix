//! UI Code for TMIX
use std::{
    io::{self, Result},
    sync::mpsc::Receiver,
    thread::{self, sleep},
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
use tmix::{data::DataManager, pulse_api::{SinkInputInformation, PulseAPI}};

const APP_NAME: &str = "TMIX";

/// Application Manager For TMIX
pub struct App {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    api: PulseAPI
}

pub enum UIMessages {
    Shutdown,
    Tick,
}

impl App {
    pub fn try_new() -> Result<Self> {
        let mut api = PulseAPI::new();
        api.startup_connection()?;
        Ok(Self {
            terminal: None,
            api: api,
        })
    }

    /// Launch Terminal Process and begin Listening for events
    pub fn run(&mut self) -> Result<()> {
        // Setup a Main Loop
        self.start_up_tui()?;
        // Do Some Thing!
        let now = SystemTime::now();
        while now.elapsed().expect("Something broke in time") < Duration::from_millis(15000) {
            let data = self.api.get_sink_inputs()?;
            self.draw_data(data.take())?;
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

    fn draw_data(&mut self, data: Vec<SinkInputInformation>) -> Result<()> {
        self.terminal
            .as_mut()
            .expect("don't draw till intialized")
            .draw(|f| {
                let size = f.size();
                let block = Block::default().title(APP_NAME).borders(Borders::ALL);
                f.render_widget(block, size);
                ui(f, &data);
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

/// Module for defining UI stuff
mod ui {
    use std::collections::hash_map::Values;

    use log::debug;
    use pulse::volume::VolumeLinear;
    use tmix::{data::DataManager, pulse_api::SinkInputInformation};
    use tui::{
        backend::Backend,
        buffer::Buffer,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Style},
        symbols,
        widgets::{Block, Borders, Widget},
        Frame,
    };

    pub(crate) fn ui<B: Backend>(f: &mut Frame<B>, data: &Vec<SinkInputInformation>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    // Eventually We will want to be able to handle more than 5 sliders
                    // But I think we will alwyas only want 5 on screen at a time.
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(f.size());
        for (i, info) in data.iter().enumerate() {
            let block = Block::default()
                .title(format!(
                    "{}",
                    info.name.as_ref().unwrap_or(&format!("Window {i}"))
                ))
                .borders(Borders::ALL);
            let bar = VolumeMeter::default()
                .block(block)
                .value((Into::<VolumeLinear>::into(info.volume.avg()).0 * 100.0) as u8);
            f.render_widget(bar, *chunks.get(i).expect("Testing for now"));
        }
    }

    #[derive(Debug, Default)]
    struct VolumeMeter<'a> {
        value: u8,
        block: Option<Block<'a>>,
    }

    impl<'a> VolumeMeter<'a> {
        /// Set the block to embed the VolumeMeter in
        pub fn block(mut self, b: Block<'a>) -> Self {
            self.block = Some(b);
            self
        }

        /// Set the Value of the Meter, Values > 100 will be set to 100
        pub fn value(mut self, val: u8) -> Self {
            self.value = val;
            self
        }
    }

    impl<'a> Widget for VolumeMeter<'a> {
        fn render(mut self, area: Rect, buf: &mut Buffer) {
            buf.set_style(area, Style::default().fg(Color::DarkGray));

            // Get the Meter Area
            let meter_area = match self.block.take() {
                Some(b) => {
                    let inner_area = b.inner(area);
                    b.render(area, buf);
                    inner_area
                }
                None => area,
            };

            // Not Enough space to draw
            if meter_area.height < 2 {
                return;
            }

            let max = 100;
            let min = 0;

            // Get the center of the cell
            let center = meter_area.left() + meter_area.width / 2;
            let top = meter_area.top() + 1;
            let bottom = meter_area.height - 1;
            let value_pos = bottom - ((bottom - top) * self.value as u16 / 100);

            // Draw the Meter
            for vert in top..=bottom {
                let mut fg_color = Color::DarkGray;
                let symbol = if vert == value_pos {
                    fg_color = Color::Gray;
                    buf.get_mut(center - 1, vert)
                        .set_symbol(symbols::bar::FULL)
                        .set_fg(fg_color);
                    buf.get_mut(center + 1, vert)
                        .set_symbol(symbols::bar::FULL)
                        .set_fg(fg_color);
                    symbols::bar::FULL
                } else if vert == top {
                    symbols::line::THICK.horizontal_down
                } else if vert == bottom {
                    symbols::line::THICK.horizontal_up
                }
                // TODO: Rather than do this modulo style, maybe we have it split in half until the
                // space between eahc split is < 5. Then use that as the distance spread.
                else if vert % (5) == 0 {
                    symbols::line::THICK.cross
                } else {
                    symbols::line::THICK.vertical
                };
                buf.get_mut(center, vert)
                    .set_symbol(symbol)
                    .set_fg(fg_color);
            }

            buf.set_stringn(
                meter_area.left() + 1,
                meter_area.bottom() - 1,
                format!("{}%", self.value),
                5,
                Style::default(),
            );
        }
    }
}
