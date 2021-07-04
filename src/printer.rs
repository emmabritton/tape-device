use std::cell::RefCell;
use std::rc::Rc;

pub type RcBox<T> = Rc<RefCell<Box<T>>>;

pub trait Printer {
    fn print(&mut self, msg: &str);
    fn eprint(&mut self, msg: &str);
    fn newline(&mut self);
    fn output(&self) -> String;
    fn error_output(&self) -> String;
    fn lines(&self) -> usize;
}

pub struct StdoutPrinter;

impl StdoutPrinter {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> RcBox<dyn Printer> {
        Rc::new(RefCell::new(Box::new(StdoutPrinter {})))
    }
}

impl Printer for StdoutPrinter {
    fn print(&mut self, msg: &str) {
        print!("{}", msg);
    }

    fn eprint(&mut self, msg: &str) {
        eprintln!("{}", msg);
    }

    fn newline(&mut self) {
        println!()
    }

    fn output(&self) -> String {
        unimplemented!()
    }

    fn error_output(&self) -> String {
        unimplemented!()
    }

    fn lines(&self) -> usize {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct DebugPrinter {
    pub output: String,
    pub error_output: String,
    pub lines: usize,
}

impl DebugPrinter {
    #[allow(clippy::new_ret_no_self)]
    #[allow(dead_code)]
    pub fn new() -> RcBox<dyn Printer> {
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
        self.output.push('\n');
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
