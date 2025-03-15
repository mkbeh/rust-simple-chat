type CloserFunc<'a> = Box<dyn Fn() + 'a>;

#[derive(Default)]
pub struct Closer<'a> {
    closers: Vec<CloserFunc<'a>>,
}

impl<'a> Closer<'a> {
    pub fn push(&mut self, callback: CloserFunc<'a>) {
        self.closers.push(callback);
    }
    pub fn close(&mut self) {
        self.closers.iter_mut().for_each(|cb| cb());
    }
}
