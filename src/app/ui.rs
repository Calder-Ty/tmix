//! UI Functions

use pulse::volume::VolumeLinear;
use tmix::{data::SinkInputInformation, pulse_api::VolumeInfo};
use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Widget},
    Frame,
};

pub(crate) fn ui<B: Backend>(f: &mut Frame<B>, data: VolumeInfo) {
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
    for (chunk, (i, info)) in data.iter().enumerate() {

        let sink_volume = (Into::<VolumeLinear>::into(info.sink().volume.avg()).0);

        let block = Block::default()
            .title(format!(
                "{}",
                info.sink().name.as_ref().unwrap_or(&format!("Window {i}"))
            ))
            .borders(Borders::ALL);
        let bar = VolumeMeter::default()
            .block(block)
            .value((sink_volume * 100.0) as u8);
        f.render_widget(bar, *chunks.get(chunk).expect("Testing for now"));

        let mut count = chunk + 1;

        for input in info.iter() {
            let input_volume = (Into::<VolumeLinear>::into(input.volume.avg()).0 * 100.0) * sink_volume;
            let block = Block::default()
                .title(format!(
                    "{}",
                    input.name.as_ref().unwrap_or(&format!("Window {i}"))
                ))
                .borders(Borders::ALL);
            let bar = VolumeMeter::default()
                .block(block)
                .value(input_volume as u8);
            f.render_widget(bar, *chunks.get(count).expect("Testing for now"));
            count += 1;
        }

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
