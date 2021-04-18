use crate::printer::{Printer, RcBox};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct DebugPrinter {
    pub output: String,
    pub error_output: String,
    pub lines: usize,
}

impl DebugPrinter {
    pub(super) fn new() -> RcBox<dyn Printer> {
        Rc::new(RefCell::new(Box::new(DebugPrinter::default())))
    }
}

impl Printer for DebugPrinter {
    fn print(&mut self, msg: &str) {
        self.output.push_str(msg);
    }

    fn eprint(&mut self, msg: &str) {
        self.error_output.push_str(msg);
    }

    fn newline(&mut self) {
        self.output.push_str("\n");
        self.lines += 1;
    }

    fn output(&self) -> String {
        self.output.clone()
    }

    fn error_output(&self) -> String {
        self.error_output.clone()
    }

    fn lines(&self) -> usize {
        self.lines
    }
}
