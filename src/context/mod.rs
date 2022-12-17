use super::*;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Context {
    cache: HashMap<Instruction, Table>,
}

impl Context {
    pub fn check_equivalence(&mut self, a: &Instruction, b: &Instruction) -> bool {
        if !self.cache.contains_key(a) {
            let ev = a.evaluate().expect("internal context should be infallible");
            self.cache.insert(a.clone(), ev);
        }

        if !self.cache.contains_key(b) {
            let ev = b.evaluate().expect("internal context should be infallible");
            self.cache.insert(b.clone(), ev);
        }

        let a = self.cache.get(a).expect("key was checked");
        let b = self.cache.get(b).expect("key was checked");

        a == b
    }
}
