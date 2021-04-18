use crate::debugger::internals::Debugger;
use crate::debugger::printer::DebugPrinter;
use crate::decompiler::{collect_jump_targets, decode, Decoded};
use crate::device::internals::Device;
use crate::printer::{Printer, RcBox};
use crate::tape_reader::{read_tape, Tape};
use anyhow::Result;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use scopeguard::defer;
use std::io::{stdout, Stdout};
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod internals;
mod printer;
mod status_widget;

type DebugTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn start(path: &str, input_path: Option<&str>) -> Result<()> {
    let printer = DebugPrinter::new();
    let tape = read_tape(path)?;
    let decoded = decompile(&tape);
    let (name, version, device) = create_device(tape, input_path, printer.clone())?;

    let mut terminal = create_terminal()?;
    defer! {
        close_terminal().expect("Failed to close terminal");
    }

    let title = format!("{} v{}", name, version);
    let mut debugger = Debugger::new(title, device, decoded, printer);
    debugger.run(&mut terminal)?;

    Ok(())
}

fn create_terminal() -> Result<DebugTerminal> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    Ok(terminal)
}

fn close_terminal() -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn decompile(tape: &Tape) -> Vec<Decoded> {
    let jump_targets = collect_jump_targets(&tape.ops);
    let mut results = vec![];
    for (idx, op) in tape.ops.iter().enumerate() {
        results.push(decode(op, &tape.data, idx, jump_targets.contains(&idx)));
    }
    results
}

fn create_device(
    tape: Tape,
    input_path: Option<&str>,
    printer: RcBox<dyn Printer>,
) -> Result<(String, String, Device)> {
    Ok((
        tape.name,
        tape.version,
        Device::new(
            tape.ops,
            tape.data,
            input_path.map(|str| str.to_string()),
            printer,
        ),
    ))
}
