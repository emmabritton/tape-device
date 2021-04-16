pub trait Printer {
    fn print(&self, msg: &str);
    fn eprint(&self, msg: &str);
    fn newline(&self);
}

pub struct StdoutPrinter;

impl StdoutPrinter {
    pub fn boxed() -> Box<Self> {
        Box::new(StdoutPrinter {})
    }
}

impl Printer for StdoutPrinter {
    fn print(&self, msg: &str) {
        print!("{}", msg);
    }

    fn eprint(&self, msg: &str) {
        eprintln!("{}", msg);
    }

    fn newline(&self) {
        println!()
    }
}
