use crate::assembler::debug_model::DebugModel;
use crate::constants::code::RET;
use crate::constants::{get_addr_byte_offset, is_jump_op};
use crate::device::comm::Output;
use crate::device::internals::{Device, RunResult};
use crate::device::util::{convert_and_fit, fit_in_lines};
use crate::device::Dump;
use anyhow::Result;
use crossterm::cursor::{Hide, MoveToColumn, MoveToPreviousLine, Show};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::style::Styler;
use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;
use std::io::stdout;
use std::thread::sleep;
use std::time::Duration;

pub struct DebugDevice {
    device: Device,
    debug: DebugModel,
    last_run_result: RunResult,
    ui_memory: Option<(u16, u16)>,
    state: DebuggerState,
    footer_height: u16,
    redraw: bool,
    hex_8bit: bool,
    hex_16bit: bool,
    dump_chars: bool,
    original_line: bool,
    print_info: bool,
    print_help: bool,
    print_history: bool,
    auto_run: bool,
    history: Vec<Option<History>>,
}

#[derive(Debug)]
struct History {
    line_num: usize,
    byte_addr: u16,
    bytes: Vec<u8>,
    line: String,
    previous: PreviousType,
}

#[derive(Debug, PartialEq)]
enum PreviousType {
    Other,
    Jump,
    Return,
}

pub fn setup_terminal() -> Result<()> {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        shutdown_terminal();
        default_panic(info);
    }));

    crossterm::terminal::enable_raw_mode()?;
    stdout().execute(Hide)?;

    Ok(())
}

pub fn shutdown_terminal() {
    let result = crossterm::terminal::disable_raw_mode();
    if let Err(err) = result {
        eprintln!(
            "Failed to disable raw mode, you'll need to close this terminal window:\n{}",
            err
        );
    }
    let mut stdout = stdout();
    let result = stdout.execute(Show);
    if let Err(err) = result {
        eprintln!(
            "Failed to restore cursor, you'll need to close this terminal window:\n{}",
            err
        );
    }
    println!("\n\n");
}

impl DebugDevice {
    pub fn new(
        ops: Vec<u8>,
        strings: Vec<u8>,
        data: Vec<u8>,
        debug_info: DebugModel,
        data_files: Vec<String>,
    ) -> Self {
        DebugDevice {
            device: Device::new(ops, strings, data, data_files),
            debug: debug_info,
            last_run_result: RunResult::Pause,
            ui_memory: None,
            state: DebuggerState::Ready,
            footer_height: 0,
            redraw: true,
            hex_8bit: true,
            hex_16bit: true,
            dump_chars: false,
            original_line: false,
            print_info: false,
            print_help: false,
            print_history: false,
            auto_run: false,
            history: vec![],
        }
    }
}

impl DebugDevice {
    pub fn run(&mut self) -> Result<()> {
        println!();
        println!();
        loop {
            self.draw()?;
            if let Some(input) = self.process_input()? {
                match input {
                    Input::ForceStep => {
                        if self.state == DebuggerState::Ready {
                            let pc = self.device.pc;
                            self.last_run_result = self.device.step(true);
                            if should_add_history(&self.last_run_result) && pc != self.device.pc {
                                self.add_history(pc);
                            }
                        }
                    }
                    Input::SetBreakpoint(byte) => self.device.breakpoints.push(byte),
                    Input::ClearBreakpoint(byte) => {
                        remove_if_present(&mut self.device.breakpoints, &byte)
                    }
                    Input::Char(chr) => {
                        self.device.keyboard_buffer.push(chr as u8);
                        self.state = DebuggerState::Ready;
                        if self.last_run_result == RunResult::CharInputRequested {
                            let pc = self.device.pc;
                            self.last_run_result = self.device.step(true);
                            if should_add_history(&self.last_run_result) && pc != self.device.pc {
                                self.add_history(pc);
                            }
                        }
                    }
                    Input::Text(str) => {
                        self.device
                            .keyboard_buffer
                            .extend_from_slice(str.as_bytes());
                        self.state = DebuggerState::Ready;
                        let pc = self.device.pc;
                        self.last_run_result = self.device.step(true);
                        if should_add_history(&self.last_run_result) && pc != self.device.pc {
                            self.add_history(pc);
                        }
                    }
                    Input::Terminate => return Ok(()),
                    Input::Toggle16BitDisplay => self.hex_16bit = !self.hex_16bit,
                    Input::Toggle8BitDisplay => self.hex_8bit = !self.hex_8bit,
                    Input::ToggleDumpCharacters => self.dump_chars = !self.dump_chars,
                    Input::ToggleOriginalLine => self.original_line = !self.original_line,
                    Input::Info => self.print_info = true,
                    Input::Help => self.print_help = true,
                    Input::ExecutionHistory => self.print_history = true,
                    Input::ToggleAutoRun => self.auto_run = !self.auto_run,
                }
                self.redraw = true;
            }

            match self.last_run_result {
                RunResult::Pause => {
                    if self.auto_run {
                        let pc = self.device.pc;
                        self.last_run_result = self.device.step(false);
                        if self.last_run_result != RunResult::Pause {
                            self.auto_run = false;
                        }
                        if should_add_history(&self.last_run_result) && pc != self.device.pc {
                            self.add_history(pc);
                        }
                        self.redraw = true;
                        sleep(Duration::from_millis(1));
                    }
                }
                RunResult::Breakpoint => {
                    self.state = DebuggerState::Ready;
                }
                RunResult::EoF | RunResult::Halt | RunResult::ProgError => {
                    self.state = DebuggerState::ProgEnd;
                    self.redraw = true;
                }
                RunResult::CharInputRequested => {
                    if self.state != DebuggerState::WaitingForChar {
                        self.state = DebuggerState::WaitingForChar;
                        self.redraw = true;
                    }
                }
                RunResult::StringInputRequested => {
                    if let DebuggerState::WaitingForString(_) = self.state {
                    } else {
                        self.state = DebuggerState::WaitingForString(String::new());
                        self.redraw = true;
                    }
                }
            }
        }
    }

    fn process_input(&mut self) -> Result<Option<Input>> {
        if crossterm::event::poll(Duration::from_millis(10)).unwrap_or(false) {
            match crossterm::event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return Ok(Some(Input::Terminate));
                    }
                    if key.code == KeyCode::Esc && key.modifiers == KeyModifiers::empty() {
                        match self.state {
                            DebuggerState::Ready => {
                                if self.auto_run {
                                    self.auto_run = false;
                                } else {
                                    return Ok(Some(Input::Terminate));
                                }
                            }
                            _ => self.state = DebuggerState::Ready,
                        }
                        self.redraw = true;
                        return Ok(None);
                    }

                    match &self.state {
                        DebuggerState::Ready => {
                            if let KeyCode::Char('b') = key.code {
                                self.state =
                                    DebuggerState::WaitingForBreakpointLineToSet(String::new());
                                self.redraw = true;
                                return Ok(None);
                            } else if let KeyCode::Char('u') = key.code {
                                self.state =
                                    DebuggerState::WaitingForBreakpointLineToClear(String::new());
                                self.redraw = true;
                                return Ok(None);
                            } else if let KeyCode::Char('t') = key.code {
                                self.state = DebuggerState::WaitingForChar;
                                self.redraw = true;
                                return Ok(None);
                            } else if let KeyCode::Char('s') = key.code {
                                self.state = DebuggerState::WaitingForString(String::new());
                                self.redraw = true;
                                return Ok(None);
                            }
                            let input = match key.code {
                                KeyCode::Char(' ') => Some(Input::ForceStep),
                                KeyCode::Char('8') => Some(Input::Toggle8BitDisplay),
                                KeyCode::Char('6') => Some(Input::Toggle16BitDisplay),
                                KeyCode::Char('c') => Some(Input::ToggleDumpCharacters),
                                KeyCode::Char('l') => Some(Input::ToggleOriginalLine),
                                KeyCode::Char('i') => Some(Input::Info),
                                KeyCode::Char('h') => Some(Input::Help),
                                KeyCode::Char('y') => Some(Input::ExecutionHistory),
                                KeyCode::Char('a') => Some(Input::ToggleAutoRun),
                                _ => None,
                            };
                            return Ok(input);
                        }
                        DebuggerState::WaitingForChar => {
                            return match key.code {
                                KeyCode::Char(chr) => Ok(Some(Input::Char(chr))),
                                KeyCode::Enter => Ok(Some(Input::Char(10_u8 as char))),
                                KeyCode::Tab => Ok(Some(Input::Char(9_u8 as char))),
                                KeyCode::Backspace => Ok(Some(Input::Char(8_u8 as char))),
                                KeyCode::Esc => Ok(Some(Input::Char(27_u8 as char))),
                                KeyCode::Delete => Ok(Some(Input::Char(127_u8 as char))),
                                _ => Ok(None),
                            }
                        }
                        DebuggerState::WaitingForString(line) => {
                            match key.code {
                                KeyCode::Char(chr) => {
                                    let mut text = line.clone();
                                    text.push(chr);
                                    self.state = DebuggerState::WaitingForString(text);
                                    self.redraw = true;
                                }
                                KeyCode::Enter => {
                                    self.redraw = true;
                                    return Ok(Some(Input::Text(line.clone())));
                                }
                                KeyCode::Backspace => {
                                    if !line.is_empty() {
                                        let mut new_line = line.clone();
                                        new_line.truncate(line.len() - 1);
                                        self.state = DebuggerState::WaitingForString(new_line);
                                        self.redraw = true;
                                    }
                                }
                                _ => {}
                            }
                            return Ok(None);
                        }
                        DebuggerState::WaitingForBreakpointLineToSet(line) => {
                            match key.code {
                                KeyCode::Char(chr) => {
                                    if ('0'..='9').contains(&chr) {
                                        let mut num = line.clone();
                                        num.push(chr);
                                        self.state =
                                            DebuggerState::WaitingForBreakpointLineToSet(num);
                                        self.redraw = true;
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Ok(num) = line.parse::<usize>() {
                                        let byte_addr = self.debug.byte_for_line(num);
                                        self.state = DebuggerState::Ready;
                                        if let Some(addr) = byte_addr {
                                            return Ok(Some(Input::SetBreakpoint(addr)));
                                        } else {
                                            eprintln!("No op on that line");
                                        }
                                    } else {
                                        eprintln!("Invalid line num")
                                    }
                                    self.redraw = true;
                                }
                                KeyCode::Backspace => {
                                    if !line.is_empty() {
                                        let mut new_line = line.clone();
                                        new_line.truncate(line.len() - 1);
                                        self.state =
                                            DebuggerState::WaitingForBreakpointLineToSet(new_line);
                                        self.redraw = true;
                                    }
                                }
                                _ => {}
                            }
                            return Ok(None);
                        }
                        DebuggerState::WaitingForBreakpointLineToClear(line) => {
                            match key.code {
                                KeyCode::Char(chr) => {
                                    if ('0'..='9').contains(&chr) {
                                        let mut num = line.clone();
                                        num.push(chr);
                                        self.state =
                                            DebuggerState::WaitingForBreakpointLineToClear(num);
                                        self.redraw = true;
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Ok(num) = line.parse::<usize>() {
                                        let byte_addr = self.debug.byte_for_line(num);
                                        self.state = DebuggerState::Ready;
                                        if let Some(addr) = byte_addr {
                                            return Ok(Some(Input::ClearBreakpoint(addr)));
                                        } else {
                                            eprintln!("No op on that line");
                                        }
                                    } else {
                                        eprintln!("Invalid line num")
                                    }
                                    self.redraw = true;
                                }
                                KeyCode::Backspace => {
                                    if !line.is_empty() {
                                        let mut new_line = line.clone();
                                        new_line.truncate(line.len() - 1);
                                        self.state = DebuggerState::WaitingForBreakpointLineToClear(
                                            new_line,
                                        );
                                        self.redraw = true;
                                    }
                                }
                                _ => {}
                            }
                            return Ok(None);
                        }
                        DebuggerState::ProgEnd => {
                            let input = match key.code {
                                KeyCode::Char('8') => Some(Input::Toggle8BitDisplay),
                                KeyCode::Char('6') => Some(Input::Toggle16BitDisplay),
                                KeyCode::Char('c') => Some(Input::ToggleDumpCharacters),
                                KeyCode::Char('l') => Some(Input::ToggleOriginalLine),
                                KeyCode::Char('i') => Some(Input::Info),
                                KeyCode::Char('h') => Some(Input::Help),
                                KeyCode::Char('y') => Some(Input::ExecutionHistory),
                                _ => None,
                            };
                            return Ok(input);
                        }
                    }
                }
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
            }
        }
        Ok(None)
    }

    fn get_last_op_type(&self) -> PreviousType {
        if !self.history.is_empty() && self.history.last().unwrap().is_some() {
            let prev_op = self.history.last().unwrap().as_ref().unwrap().bytes[0];
            //Check RET first as is_jump_op returns true for RET
            //but we want to treat RET differently in the UI
            if prev_op == RET {
                PreviousType::Return
            } else if is_jump_op(prev_op) {
                PreviousType::Jump
            } else {
                PreviousType::Other
            }
        } else {
            PreviousType::Other
        }
    }

    fn add_history(&mut self, addr: u16) {
        let op = self.debug.op_for_byte(addr);
        if let Some(op) = op {
            let last_op_type = self.get_last_op_type();
            self.history.push(Some(History {
                line_num: op.line_num,
                byte_addr: op.byte_addr,
                bytes: op.bytes.clone(),
                line: op.processed_line.clone(),
                previous: last_op_type,
            }));
        } else {
            self.history.push(None);
        }
    }

    fn draw(&mut self) -> Result<()> {
        if self.redraw {
            self.reset_cursor()?;
            let mut newline_printed = false;
            if !self.device.output.is_empty() {
                for output in &self.device.output {
                    match output {
                        Output::OutputStd(str) => {
                            print!("{}", str);
                            newline_printed = str.ends_with('\n');
                        }
                        Output::OutputErr(err) => {
                            stdout().execute(MoveToColumn(0))?;
                            eprintln!("Error: {}", err)
                        }
                        Output::BreakpointHit(_) => {}
                    }
                }
                if !newline_printed {
                    //print a newline after the previous text otherwise the footer will draw over it
                    //but only print one if the program didn't otherwise extra blank lines will be introduced
                    println!();
                }
            }
            self.device.output.clear();
            if self.print_history {
                println!("{}", "  Line  Addr  Bytes             Src".bold());
                for op in &self.history {
                    stdout().execute(MoveToColumn(0))?;
                    if let Some(op) = op {
                        let jmp = match op.previous {
                            PreviousType::Other => ' ',
                            PreviousType::Jump => '>',
                            PreviousType::Return => '<',
                        };
                        let bytes = format_instruction(&op.bytes, true, true, false);
                        println!(
                            "{} {: <5} {:04X}  {: <17} {}",
                            jmp, op.line_num, op.byte_addr, bytes, op.line
                        );
                    } else {
                        println!("??? Unknown instruction was executed");
                    }
                }
            }
            let (cols, _) = crossterm::terminal::size()?;
            let mut footer = self.gen_footer(cols as usize)?;
            if let Some((start, end)) = self.ui_memory {
                let bytes = self.device.mem[start as usize..end as usize]
                    .iter()
                    .map(|byte| format_8bit(*byte, self.hex_8bit, self.dump_chars))
                    .collect::<Vec<String>>();
                footer.extend_from_slice(&fit_in_lines(bytes, cols as usize));
            }
            if self.print_info {
                footer.extend_from_slice(&convert_and_fit(
                    vec![
                        &format!("8bit values as hex: {}", self.hex_8bit),
                        &format!("16bit values as hex: {}", self.hex_16bit),
                        &format!("show ASCII for registers: {}", self.dump_chars),
                        &format!("show original line: {}", self.original_line),
                    ],
                    cols as usize,
                    "   ",
                ));
            }
            if self.print_help {
                footer.extend_from_slice(&convert_and_fit(
                    vec![
                        "space) Step",
                        "ctrl+c) quit",
                        "esc) leave text entry modes or stop auto-run",
                        "8) Toggle dec/hex for 8bit",
                        "6) Toggle dec/hex for 16it",
                        "i) Print debugger info",
                        "c) Toggle printing ASCII chars",
                        "l) Toggle printing original line",
                        "b) Set breakpoint",
                        "u) Clear breakpoint",
                        "h) Print help",
                        "y) Print execution history",
                        "t) Input char",
                        "s) Input string",
                    ],
                    cols as usize,
                    "   ",
                ));
            }
            self.footer_height = footer.len() as u16;
            for line in footer {
                stdout().execute(MoveToColumn(0))?;
                println!("{}", line);
            }
            self.redraw = false;
            self.print_info = false;
            self.print_help = false;
            self.print_history = false
        }
        Ok(())
    }

    fn gen_footer(&self, cols: usize) -> Result<Vec<String>> {
        let mut lines = vec![];
        lines.push(format!("{1:-<0$}", cols, "-"));
        let status_value = match (&self.last_run_result, &self.state) {
            (RunResult::Pause, DebuggerState::Ready) => String::from("Ready"),
            (RunResult::Pause, DebuggerState::WaitingForBreakpointLineToSet(num)) => format!(
                "Enter breakpoint line num to set (esc to cancel): {}              ",
                num
            ),
            (RunResult::Pause, DebuggerState::WaitingForBreakpointLineToClear(num)) => format!(
                "Enter breakpoint line num to clear (esc to cancel): {}            ",
                num
            ),
            (RunResult::StringInputRequested, DebuggerState::WaitingForString(str)) => {
                format!("Enter a string (and then press enter): {}", str)
            }
            (RunResult::StringInputRequested, DebuggerState::Ready) => {
                String::from("Waiting for string input")
            }
            (RunResult::Pause, DebuggerState::WaitingForChar) => {
                String::from("Press any valid key")
            }
            (RunResult::CharInputRequested, DebuggerState::WaitingForChar) => {
                String::from("Press any valid key")
            }
            (RunResult::CharInputRequested, DebuggerState::Ready) => {
                String::from("Waiting for character input")
            }
            (RunResult::Breakpoint, _) => String::from("Breakpoint Hit"),
            (RunResult::ProgError, _) => String::from("Crashed"),
            (_, _) => String::from("End of Program"),
        };
        let op = self.debug.op_for_byte(self.device.pc);
        let line_text = if let Some(op) = op {
            format!(
                "Bytes: {: <17}     Lines: {}",
                format_instruction(&op.bytes, self.hex_8bit, self.hex_16bit, self.dump_chars),
                if self.original_line {
                    op.original_line.trim().to_string()
                } else {
                    op.processed_line.clone()
                }
            )
        } else {
            String::from("Unknown line: ???")
        };
        lines.push(format!("Status: {}     {}", status_value, line_text));
        let mut dump = format_dump(
            self.device.dump(),
            self.hex_8bit,
            self.hex_16bit,
            self.dump_chars,
        );
        let line_num = op
            .map(|op| op.line_num.to_string())
            .unwrap_or_else(|| String::from("??"));
        dump.insert(0, format!("Line Num: {: <5}  ", line_num));
        lines.extend_from_slice(&fit_in_lines(dump, cols as usize - 1));
        Ok(lines)
    }

    fn reset_cursor(&self) -> Result<()> {
        stdout().execute(MoveToPreviousLine(self.footer_height))?;
        stdout().execute(Clear(ClearType::FromCursorDown))?;
        Ok(())
    }
}

fn format_dump(dump: Dump, hex_8bit: bool, hex_16bit: bool, chars: bool) -> Vec<String> {
    vec![
        format!("PC: {}  ", format_16bit(dump.pc, hex_16bit, false)),
        format!("SP: {}  ", format_16bit(dump.sp, hex_16bit, chars)),
        format!("FP: {}  ", format_16bit(dump.fp, hex_16bit, chars)),
        format!("A0: {}  ", format_16bit(dump.addr_reg[0], hex_16bit, chars)),
        format!("A1: {}  ", format_16bit(dump.addr_reg[1], hex_16bit, chars)),
        format!("ACC: {}  ", format_8bit(dump.acc, hex_8bit, chars)),
        format!("D0: {}  ", format_8bit(dump.data_reg[0], hex_8bit, chars)),
        format!("D1: {}  ", format_8bit(dump.data_reg[1], hex_8bit, chars)),
        format!("D2: {}  ", format_8bit(dump.data_reg[2], hex_8bit, chars)),
        format!("D3: {}  ", format_8bit(dump.data_reg[3], hex_8bit, chars)),
        format!("Overflow: {}  ", dump.overflow),
    ]
}

fn format_instruction(bytes: &[u8], hex_8bit: bool, hex_16bit: bool, chars: bool) -> String {
    let mut iter = bytes.iter().peekable();
    let op = iter.next().expect("No OP byte found for instruction");
    let mut output = format!("{} ", format_8bit(*op, true, false));
    let offset = get_addr_byte_offset(*op).unwrap_or(0);
    let mut idx = 1;
    loop {
        if !output.ends_with(' ') {
            output.push(' ');
        }
        if iter.peek().is_none() {
            break;
        }
        if idx == offset {
            idx += 1;
            let bytes = [*iter.next().unwrap(), *iter.next().unwrap()];
            let value = u16::from_be_bytes(bytes);
            output.push_str(&format_16bit(value, hex_16bit, chars));
        } else {
            output.push_str(&format_8bit(*iter.next().unwrap(), hex_8bit, chars));
        }
        idx += 1;
    }
    output
}

fn format_16bit(value: u16, hex: bool, chars: bool) -> String {
    let str = if hex {
        format!("{:04X}", value)
    } else {
        format!("{: >5}", value)
    };
    if chars {
        let bytes = value.to_be_bytes();
        format!("{} '{}{}'", str, u8_char(bytes[0]), u8_char(bytes[1]))
    } else {
        str
    }
}

fn format_8bit(value: u8, hex: bool, chars: bool) -> String {
    let str = if hex {
        format!("{:02X}", value)
    } else {
        format!("{: >3}", value)
    };
    if chars {
        format!("{} '{}'", str, u8_char(value))
    } else {
        str
    }
}

fn u8_char(value: u8) -> char {
    if value.is_ascii_graphic() {
        value as char
    } else {
        '□'
    }
}

fn remove_if_present<T: PartialEq>(list: &mut Vec<T>, item: &T) {
    if let Some(idx) = list.iter().position(|element| element == item) {
        list.remove(idx);
    }
}

fn should_add_history(run_result: &RunResult) -> bool {
    matches!(
        run_result,
        RunResult::Pause | RunResult::StringInputRequested | RunResult::CharInputRequested
    )
}

enum Input {
    ForceStep,
    SetBreakpoint(u16),
    ClearBreakpoint(u16),
    Char(char),
    Text(String),
    Terminate,
    Toggle16BitDisplay,
    Toggle8BitDisplay,
    ToggleDumpCharacters,
    ToggleOriginalLine,
    Info,
    Help,
    ExecutionHistory,
    ToggleAutoRun,
}

#[derive(Debug, PartialEq)]
enum DebuggerState {
    Ready,
    WaitingForBreakpointLineToSet(String),
    WaitingForBreakpointLineToClear(String),
    WaitingForChar,
    WaitingForString(String),
    ProgEnd,
}
