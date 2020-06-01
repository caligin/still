use std::fmt::{Debug, Error, Formatter};
// pub type Search = (Vec<Box<SearchTerm>>, Vec<Box<Transform>>, Box<Option<Sort>>);
pub type Search<'input> = (Vec<SearchTerm<'input>>, Vec<Vec<&'input str>>, Option<&'input str>);

#[derive(PartialEq)]
pub enum SearchTerm<'input> {
    Include(&'input str),
    Exclude(&'input str),
    Any(),
    // Error,
}

pub enum Transform {
    Parse(),
    Filter(),
    Aggreagate(),
    // Error,
}

pub enum Sort {
    Asc(),
    // Error,
}

impl<'input> Debug for SearchTerm<'input> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::SearchTerm::*;
        match *self {
            Include(term) => write!(fmt, "{:?}", term),
            Exclude(term) => write!(fmt, "! {:?}", term),
            Any() => write!(fmt, "*")
            // Error => write!(fmt, "error"),
        }
    }
}