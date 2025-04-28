use crate::Result;
use crate::outline;
use fonty::Ttf;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait Shape<I>: Sized {
    fn new(ttf: &mut dyn Ttf, instance: Rc<RefCell<I>>, c: char) -> Result<Self>;
}

pub trait Instance: Sized {
    type Shape: Shape<Self>;

    fn new() -> Self;
}

pub struct Factory<'a, I: Instance> {
    ttf: &'a RefCell<dyn Ttf + 'a>,
    instance: Rc<RefCell<I>>,
    cache: HashMap<char, Rc<RefCell<I::Shape>>>,
}

impl<'a, I: Instance> Factory<'a, I> {
    pub fn new(ttf: &'a RefCell<dyn Ttf + 'a>) -> Self {
        Self {
            ttf,
            instance: Rc::new(RefCell::new(Instance::new())),
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, c: char) -> Result<Rc<RefCell<I::Shape>>> {
        if let Some(shape) = self.cache.get(&c) {
            Ok(shape.clone())
        } else {
            let shape = Rc::new(RefCell::new(I::Shape::new(
                &mut *self.ttf.borrow_mut(),
                self.instance.clone(),
                c,
            )?));

            self.cache.insert(c, shape);
            Ok(self.cache[&c].clone())
        }
    }
}

pub type OutlineFactory<'a> = Factory<'a, outline::Instance>;
