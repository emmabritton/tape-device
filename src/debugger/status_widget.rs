use crate::device::internals::Dump;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::{Block, Widget};

pub(super) struct StatusWidget<'a> {
    dump: &'a Dump,
    block: Option<Block<'a>>,
    formatter_8bit: &'a dyn Fn(u8) -> String,
    formatter_16bit: &'a dyn Fn(u16) -> String,
}

impl<'a> StatusWidget<'a> {
    pub fn new<T8, T16>(dump: &'a Dump, formatter_8bit: &'a T8, formatter_16bit: &'a T16) -> Self
    where
        T8: Fn(u8) -> String,
        T16: Fn(u16) -> String,
    {
        StatusWidget {
            dump,
            block: None,
            formatter_8bit,
            formatter_16bit,
        }
    }
}

impl<'a> StatusWidget<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for StatusWidget<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let inner_area = match self.block {
            Some(ref mut b) => {
                b.draw(area, buf);
                b.inner(area)
            }
            None => area,
        };

        buf.set_string(
            inner_area.x,
            inner_area.y,
            format!(
                "ACC: {: >3} PC: {: >4} Overflowed: {}",
                (self.formatter_8bit)(self.dump.acc),
                (self.formatter_16bit)(self.dump.pc),
                self.dump.overflow
            ),
            Style::default(),
        );
        buf.set_string(
            inner_area.x,
            inner_area.y + 1,
            format!(
                "D0: {: >3} D1: {: >3} D2: {: >3} D3: {: >3}",
                (self.formatter_8bit)(self.dump.data_reg[0]),
                (self.formatter_8bit)(self.dump.data_reg[1]),
                (self.formatter_8bit)(self.dump.data_reg[2]),
                (self.formatter_8bit)(self.dump.data_reg[3])
            ),
            Style::default(),
        );
        buf.set_string(
            inner_area.x,
            inner_area.y + 2,
            format!(
                "A0: {: >4} A1: {: >4}",
                (self.formatter_16bit)(self.dump.addr_reg[0]),
                (self.formatter_16bit)(self.dump.addr_reg[1]),
            ),
            Style::default(),
        );
        buf.set_string(
            inner_area.width / 3,
            inner_area.y,
            "<SPACE> Step <Q, ESC> Quit <H> Toggle hex/dec <O> Toggle decoded ops",
            Style::default(),
        );
        buf.set_string(
            inner_area.width / 3,
            inner_area.y + 1,
            "<UP, DOWN> Scroll ops <G> Go to current op",
            Style::default(),
        );
        buf.set_string(
            inner_area.width / 3,
            inner_area.y + 2,
            "<I, K> Scroll output <L> Go to last output",
            Style::default(),
        );
    }
}
