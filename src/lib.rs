use context::Context;
use std::{iter, ops::Deref};

mod context;
mod grammar;
mod traverser;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Instruction {
    True,
    False,
    Argument(String),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Xor(Box<Self>, Box<Self>),
    Conditional(Box<Self>, Box<Self>),
    Biconditional(Box<Self>, Box<Self>),
    Equals(Box<Self>, Box<Self>),
}

impl Instruction {
    pub fn evaluate(&self) -> Result<Table, String> {
        traverser::Evaluator::run(self).map(|ev| Evaluation::into_table(ev))
    }

    pub const fn eq_true(&self) -> bool {
        matches!(self, Self::True)
    }

    pub const fn eq_false(&self) -> bool {
        matches!(self, Self::False)
    }

    pub const fn is_not(&self) -> bool {
        matches!(self, Self::Not(_))
    }

    pub const fn is_and(&self) -> bool {
        matches!(self, Self::Or(_, _))
    }

    pub const fn is_or(&self) -> bool {
        matches!(self, Self::Or(_, _))
    }

    fn _optimize(self, context: &mut Context) -> Self {
        use Instruction::*;
        match self {
            True => True,
            False => False,
            Argument(a) => Argument(a),

            Not(x) => {
                let x = x._optimize(context);

                if x.eq_true() {
                    return False;
                } else if x.eq_false() {
                    return True;
                }

                match x {
                    Not(x) => *x,
                    _ => Not(Box::new(x)),
                }
            }

            And(l, r) => {
                let l = l._optimize(context);
                let r = r._optimize(context);

                if l.eq_true() && r.eq_true() {
                    return True;
                } else if l.eq_true() {
                    return r;
                } else if r.eq_true() {
                    return l;
                } else if l.eq_false() || r.eq_false() {
                    return False;
                } else if l == r {
                    return l;
                }

                if context.check_equivalence(&l, &r) {
                    return l;
                } else if context.check_equivalence(&l, &Self::Not(Box::new(r.clone()))) {
                    return False;
                } else if context.check_equivalence(&r, &Self::Not(Box::new(l.clone()))) {
                    return False;
                }

                And(Box::new(l), Box::new(r))
            }

            Or(l, r) => {
                let l = l._optimize(context);
                let r = r._optimize(context);

                if l.eq_true() || r.eq_true() {
                    return True;
                } else if l.eq_false() {
                    return r;
                } else if r.eq_false() {
                    return l;
                } else if l == r {
                    return l;
                }

                if context.check_equivalence(&l, &r) {
                    return l;
                } else if context.check_equivalence(&l, &Self::Not(Box::new(r.clone()))) {
                    return True;
                } else if context.check_equivalence(&r, &Self::Not(Box::new(l.clone()))) {
                    return True;
                }

                if let And(a, b) = &l {
                    if context.check_equivalence(&r, &a) || context.check_equivalence(&r, &b) {
                        return r;
                    }
                }

                if let And(a, b) = &r {
                    if context.check_equivalence(&l, &a) || context.check_equivalence(&l, &b) {
                        return l;
                    }
                }

                Or(Box::new(l), Box::new(r))
            }

            Xor(l, r) => {
                let l = l._optimize(context);
                let r = r._optimize(context);

                if l == r {
                    return False;
                } else if l.eq_false() {
                    return r;
                } else if r.eq_false() {
                    return l;
                } else if l.eq_true() {
                    return Not(Box::new(r));
                } else if r.eq_true() {
                    return Not(Box::new(l));
                }

                let a = Or(Box::new(l.clone()), Box::new(r.clone()));
                let b = And(Box::new(l), Box::new(r));
                let b = Not(Box::new(b));

                And(Box::new(a), Box::new(b))._optimize(context)
            }

            Conditional(l, r) => Or(Box::new(Not(l)), r)._optimize(context),

            Biconditional(l, r) | Equals(l, r) => {
                let l = *l;
                let r = *r;

                let a = And(Box::new(l.clone()), Box::new(r.clone()));
                let b = And(Box::new(Not(Box::new(l))), Box::new(Not(Box::new(r))));

                Or(Box::new(a), Box::new(b))._optimize(context)
            }
        }
    }

    pub fn optimize(self) -> Self {
        self._optimize(&mut Context::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Evaluation<'a> {
    values: Vec<(&'a str, bool)>,
    result: bool,
}

impl<'a> Deref for Evaluation<'a> {
    type Target = [(&'a str, bool)];

    fn deref(&self) -> &Self::Target {
        self.values.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<bool>>,
}

impl<'a> Evaluation<'a> {
    pub const fn result(&self) -> bool {
        self.result
    }

    pub fn into_table(result: Vec<Self>) -> Table {
        let header = result
            .first()
            .map(|ev| {
                ev.values
                    .iter()
                    .map(|(k, _)| *k)
                    .chain(iter::once("eval"))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let mut rows = result
            .into_iter()
            .map(|ev| {
                ev.values
                    .into_iter()
                    .map(|(_, v)| v)
                    .chain(iter::once(ev.result))
                    .collect()
            })
            .collect::<Vec<_>>();

        rows.as_mut_slice().sort();

        Table { header, rows }
    }
}
