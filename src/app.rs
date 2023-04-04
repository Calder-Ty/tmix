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


    use log::{debug, error, info, warn};
    use tui::{
        backend::Backend,
        buffer::Buffer,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Style},
        symbols,
        widgets::{BarChart, Block, Borders, Widget},
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
            let block = Block::default()
                .title(format!("Window #{i}"))
                .borders(Borders::ALL);
            let bar = VolumeMeter::default().block(block).value(10u8.saturating_mul(i as u8) );
            f.render_widget(bar, chunk);
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
                    buf.get_mut(center - 1, vert).set_symbol(symbols::bar::FULL).set_fg(fg_color);
                    buf.get_mut(center + 1, vert).set_symbol(symbols::bar::FULL).set_fg(fg_color);
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
                buf.get_mut(center, vert).set_symbol(symbol).set_fg(fg_color);
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
