// TODO: move inside the 'interpreter' module

use std::{fmt::Display, rc::Rc};

use crate::class::Class;

#[derive(Debug, Clone)]
pub struct Instance {
    class: Rc<Class>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self { class }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
