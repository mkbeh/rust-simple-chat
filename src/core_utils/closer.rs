type CloserFunc<'a> = Box<dyn FnMut() + 'a>;

pub struct Closer<'a> {
    closers: Vec<CloserFunc<'a>>,
}

impl<'a> Closer<'a> {
    pub fn new() -> Closer<'a> {
        Closer { closers: vec![] }
    }

    pub fn push(&mut self, callback: CloserFunc<'a>) {
        self.closers.push(Box::new(callback));
    }

    pub fn close(&mut self) {
        self.closers.iter_mut().for_each(|cb| cb());
    }
}
