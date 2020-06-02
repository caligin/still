use std::fmt::{Debug, Error, Formatter};
// pub type Search = (Vec<Box<SearchTerm>>, Vec<Box<Transform>>, Box<Option<Sort>>);
pub type Search<'input> = (Vec<SearchTerm<'input>>, Vec<Transform<'input>>, Option<Sort<'input>>);

#[derive(PartialEq)]
pub enum SearchTerm<'input> {
    Include(&'input str),
    Exclude(&'input str),
    Any(),
    // Error,
}

#[derive(PartialEq)]
pub enum Transform<'input> {
    Aggregate(Aggregation<'input>),
    Filter { field: &'input str, comparison: Comparison, value: &'input str},
    Parse { field: &'input str, parser: &'input str, bindings: Vec<&'input str>},
    // Error,
}

#[derive(PartialEq)]
pub enum Comparison {
    Ne,
    Eq,
    Match,
}

#[derive(PartialEq)]
pub enum Aggregation<'input> {
    Count(Vec<&'input str>),
}

#[derive(PartialEq)]
pub enum Sort<'input> {
    Desc(Vec<&'input str>),
    Asc(Vec<&'input str>),
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

impl<'input> Debug for Transform<'input> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Transform::*;
        match &*self {
            Aggregate(aggregation) => write!(fmt, "! {:?}", aggregation),
            Filter { field, comparison, value} => write!(fmt, "! where {:?} {:?} {:?}", field, comparison, value),
            Parse { field, parser, bindings} => write!(fmt, "! parse {:?} {:?} {:?}", field, parser, bindings),
        }
    }
}

impl<'input> Debug for Aggregation<'input> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Aggregation::*;
        match &*self {
            Count(fields) => write!(fmt, "! count by {:?}", fields),
        }
    }
}

impl<'input> Debug for Sort<'input> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Sort::*;
        match &*self {
            Asc(field) => write!(fmt, "! sort by {:?} asc", field),
            Desc(field) => write!(fmt, "! sort by {:?}", field),
        }
    }
}

impl<'input> Debug for Comparison {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Comparison::*;
        match *self {
            Ne => write!(fmt, "!="),
            Eq => write!(fmt, "="),
            Match => write!(fmt, "match"),
        }
    }
}