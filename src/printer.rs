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
