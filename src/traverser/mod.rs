use permutator::Combination;
use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use super::*;

const STACK_SIZE: usize = 20;

pub struct Arguments<'a> {
    stack: Vec<&'a Instruction>,
}

impl<'a> From<&'a Instruction> for Arguments<'a> {
    fn from(current: &'a Instruction) -> Self {
        let mut stack = Vec::with_capacity(STACK_SIZE);
        stack.push(current);

        Self { stack }
    }
}

impl<'a> Iterator for Arguments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let current = match self.stack.pop() {
            Some(c) => c,
            None => return None,
        };

        use Instruction::*;
        match current {
            Argument(a) => Some(a.as_str()),
            True | False => self.next(),
            Not(x) => {
                self.stack.push(x);
                self.next()
            }
            And(l, r)
            | Or(l, r)
            | Xor(l, r)
            | Conditional(l, r)
            | Biconditional(l, r)
            | Equals(l, r) => {
                self.stack.push(l);
                self.stack.push(r);
                self.next()
            }
        }
    }
}

impl<'a> Arguments<'a> {
    /// Returns all possible combinations for the arguments.
    ///
    /// # Example
    ///
    /// An input `[a, b, c]` will generate the following output:
    ///
    /// []
    /// [a]
    /// [a, b]
    /// [a, c]
    /// [a, b, c]
    /// [b]
    /// [b, c]
    /// [c]
    pub fn combinations(instruction: &'a Instruction) -> Vec<Vec<&'a str>> {
        let args = Self::from(instruction).collect::<HashSet<_>>();
        let args: Vec<_> = args.into_iter().collect();

        let mut combinations = (1..=args.len())
            .flat_map(|size| {
                args.combination(size)
                    .map(|c| c.iter().map(|v| **v).collect::<Vec<_>>())
            })
            .chain(iter::once(vec![]))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        combinations.as_mut_slice().sort();
        combinations
    }
}

pub struct Context<'a> {
    values: HashMap<&'a str, bool>,
}

impl<'a> Context<'a> {
    pub fn reset_to_false(&mut self) {
        self.values.values_mut().for_each(|v| *v = false);
    }
}

impl<'a> Deref for Context<'a> {
    type Target = HashMap<&'a str, bool>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<'a> DerefMut for Context<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl<'a> FromIterator<&'a str> for Context<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        iter.into_iter().map(|arg| (arg, false)).collect()
    }
}

impl<'a> FromIterator<(&'a str, bool)> for Context<'a> {
    fn from_iter<T: IntoIterator<Item = (&'a str, bool)>>(iter: T) -> Self {
        Self {
            values: HashMap::from_iter(iter),
        }
    }
}

pub struct Evaluator;

impl Evaluator {
    pub fn run(instruction: &Instruction) -> Result<Vec<Evaluation<'_>>, String> {
        let combinations = Arguments::combinations(instruction);

        let args = Arguments::from(instruction);
        let mut context = Context::from_iter(args);

        let mut evaluations = Vec::with_capacity(combinations.len());

        for case in combinations {
            context.reset_to_false();

            for value in case {
                context
                    .insert(value, true)
                    .ok_or("the combination provided an invalid argument!")?;
            }

            let result = Self::run_with_context(instruction, &context)?;
            let mut values = context.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>();

            values.as_mut_slice().sort_by_key(|(k, _)| *k);

            evaluations.push(Evaluation { values, result });
        }

        Ok(evaluations)
    }

    pub fn run_with_context<'a>(
        instruction: &'a Instruction,
        context: &'a Context<'a>,
    ) -> Result<bool, String> {
        use Instruction::*;
        match instruction {
            True => Ok(true),
            False => Ok(false),

            Argument(arg) => context
                .get(arg.as_str())
                .ok_or_else(|| "invalid evaluator context".into())
                .copied(),

            Not(x) => Ok(!Self::run_with_context(x, context)?),

            And(l, r) => {
                if !Self::run_with_context(l, context)? {
                    return Ok(false);
                }
                Self::run_with_context(r, context)
            }

            Or(l, r) => {
                if Self::run_with_context(l, context)? {
                    return Ok(true);
                }
                Self::run_with_context(r, context)
            }

            Xor(l, r) => {
                let l = Self::run_with_context(l, context)?;
                let r = Self::run_with_context(r, context)?;
                Ok(l ^ r)
            }

            Conditional(l, r) => {
                if !Self::run_with_context(l, context)? {
                    return Ok(true);
                }
                Self::run_with_context(r, context)
            }

            Biconditional(l, r) => {
                let l = Self::run_with_context(l, context)?;
                let r = Self::run_with_context(r, context)?;
                Ok(l && r || !l && !r)
            }

            Equals(l, r) => {
                let l = Self::run_with_context(l, context)?;
                let r = Self::run_with_context(r, context)?;
                Ok(l == r)
            }
        }
    }
}
