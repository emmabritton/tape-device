use crate::debugger::status_widget::StatusWidget;
use crate::debugger::DebugTerminal;
use crate::decompiler::Decoded;
use crate::device::internals::Device;
use crate::printer::{Printer, RcBox};
use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use std::time::Duration;
use tui::layout::{Constraint, Direction, Layout, ScrollMode};
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};

pub(super) struct Debugger {
    prog_title: String,
    device: Device,
    decoded: Vec<Decoded>,
    printer: RcBox<dyn Printer>,
    op_scroll_offset: u16,
    output_scroll_offset: u16,
    show_asm: bool,
    show_hex: bool,
    run: bool,
}

impl Debugger {
    pub(super) fn new(
        prog_title: String,
        device: Device,
        decoded: Vec<Decoded>,
        printer: RcBox<dyn Printer>,
    ) -> Self {
        Debugger {
            prog_title,
            device,
            decoded,
            printer,
            op_scroll_offset: 0,
            output_scroll_offset: 0,
            show_asm: true,
            show_hex: true,
            run: false,
        }
    }
}

impl Debugger {
    pub(super) fn run(&mut self, terminal: &mut DebugTerminal) -> Result<()> {
        let title = format!("Debugging {}", self.prog_title);

        loop {
            let output = self.printer.borrow().output();
            let dump = self.device.dump();

            if crossterm::event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break;
                    } else if key.code == KeyCode::Char('o') {
                        self.show_asm = !self.show_asm;
                    } else if key.code == KeyCode::Char('h') {
                        self.show_hex = !self.show_hex;
                    } else if key.code == KeyCode::Char(' ') {
                        self.device.step();
                        self.op_scroll_offset = dump.pc.saturating_sub(10);
                    } else if key.code == KeyCode::Char('k') {
                        self.output_scroll_offset = self.output_scroll_offset.saturating_sub(1);
                    } else if key.code == KeyCode::Char('i') {
                        if (self.output_scroll_offset as usize) < self.printer.borrow().lines() - 20
                        {
                            self.output_scroll_offset = self.output_scroll_offset.saturating_add(1);
                        }
                    } else if key.code == KeyCode::Up {
                        self.op_scroll_offset = self.op_scroll_offset.saturating_sub(1);
                    } else if key.code == KeyCode::Down {
                        if (self.op_scroll_offset as usize) < self.decoded.len() - 20 {
                            self.op_scroll_offset = self.op_scroll_offset.saturating_add(1);
                        }
                    } else if key.code == KeyCode::Char('g') {
                        self.op_scroll_offset = dump.pc.saturating_sub(10);
                    } else if key.code == KeyCode::Char('r') {
                        self.run = !self.run;
                    }
                }
            }

            if self.run {
                self.device.step();
            }

            terminal.draw(|mut f| {
                let vert_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(5), Constraint::Percentage(90)].as_ref())
                    .split(f.size());
                let horz_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(vert_chunks[1]);

                StatusWidget::new(&dump, &|value| self.f8bit(value), &|value| {
                    self.f16bit(value)
                })
                .block(Block::default().title(title.as_str()).borders(Borders::ALL))
                .render(&mut f, vert_chunks[0]);

                Paragraph::new([Text::raw(output)].iter())
                    .block(Block::default().title("Output").borders(Borders::ALL))
                    .scroll_mode(ScrollMode::Tail)
                    .scroll(self.output_scroll_offset)
                    .render(&mut f, horz_chunks[0]);

                let prog = self.generate_ops_string(dump.pc as usize);
                Paragraph::new(prog.iter())
                    .block(Block::default().title("Ops").borders(Borders::ALL))
                    .scroll(self.op_scroll_offset)
                    .render(&mut f, horz_chunks[1]);
            })?;
        }

        Ok(())
    }

    fn generate_ops_string(&self, current: usize) -> Vec<Text> {
        let addr_prefix = |op: &Decoded| {
            if op.is_jump_target {
                self.f16bit(op.byte_offset as u16)
            } else {
                String::from("     ")
            }
        };
        let format_span = |active: bool, text: String| {
            if active {
                Text::styled(
                    format!("> {}\n", text),
                    Style::default().modifier(Modifier::BOLD),
                )
            } else {
                Text::raw(format!("  {}\n", text))
            }
        };

        return if self.show_asm {
            self.decoded
                .iter()
                .map(|op| {
                    let text = if op.is_param_16_bit() {
                        let param = u16::from_be_bytes([op.bytes[1], op.bytes[2]]);
                        format!(
                            "{} {:6} {}",
                            addr_prefix(&op),
                            op.strings[0],
                            self.f16bit(param)
                        )
                    } else {
                        format!(
                            "{} {:6} {: <5} {}",
                            addr_prefix(&op),
                            op.strings[0],
                            self.f8bit_str(&op.strings[1]),
                            self.f8bit_str(&op.strings[2])
                        )
                    };
                    format_span(op.byte_offset == current, text)
                })
                .collect::<Vec<Text>>()
        } else {
            self.decoded
                .iter()
                .map(|op| {
                    let addr = addr_prefix(&op);
                    let text = if op.is_param_16_bit() {
                        let param = u16::from_be_bytes([op.bytes[1], op.bytes[2]]);
                        format!(
                            "{} {}  {}",
                            addr,
                            self.f8bit(op.bytes[0]),
                            self.f16bit(param)
                        )
                    } else {
                        format!(
                            "{} {}  {}  {}",
                            addr,
                            self.f8bit(op.bytes[0]),
                            self.f8bit(op.bytes[1]),
                            self.f8bit(op.bytes[2])
                        )
                    };
                    format_span(op.byte_offset == current, text)
                })
                .collect::<Vec<Text>>()
        };
    }

    fn f8bit(&self, value: u8) -> String {
        if self.show_hex {
            format!("{:02X}", value)
        } else {
            format!("{: <3}", value)
        }
    }

    fn f16bit(&self, value: u16) -> String {
        if self.show_hex {
            format!("{:04X} ", value)
        } else {
            format!("{: <5}", value)
        }
    }

    fn f8bit_str(&self, value: &str) -> String {
        if value.is_empty() {
            return String::new();
        }
        if value.chars().any(|chr| chr.is_alphabetic()) {
            return value.to_string();
        }
        if self.show_hex {
            format!("{:02X}", value.parse::<u8>().unwrap())
        } else {
            format!("{: <3}", value)
        }
    }
}
