use crate::ast::{Search, SearchTerm, Transform, Aggregation, Sort};

pub trait Visitor<'ast> {
    fn visit_search(&mut self, search: &Search);
    fn visit_search_term(&mut self, search_term: &'ast SearchTerm<'ast>);
    fn visit_transform(&mut self, transform: &Transform);
    fn visit_aggregation(&mut self, aggregation: &Aggregation);
    fn visit_sort(&mut self, sort: &Sort);
}

pub trait Visitable<'ast, V: Visitor<'ast>> {
    fn accept(&'ast self, visitor: &mut V);
}

impl<'ast, V: Visitor<'ast>> Visitable<'ast, V> for Search<'ast> {
    fn accept(&'ast self, visitor: &mut V) {
        visitor.visit_search(self);
        let (search_terms, transforms, sort) = self;
        for term in search_terms {
            term.accept(visitor);
        }
        for transform in transforms {
            transform.accept(visitor);
        }
        for sor in sort {
            sor.accept(visitor);
        }
    }
}

impl<'ast, V: Visitor<'ast>> Visitable<'ast, V> for SearchTerm<'ast> {
    fn accept(&'ast self, visitor: &mut V) {
        visitor.visit_search_term(self);
    }
}

impl<'ast, V: Visitor<'ast>> Visitable<'ast, V> for Transform<'ast> {
    fn accept(&'ast self, visitor: &mut V) {
        visitor.visit_transform(self);
        match self {
            Transform::Aggregate(aggregation) => aggregation.accept(visitor),
            _ => (),
        }
    }
}

impl<'ast, V: Visitor<'ast>> Visitable<'ast, V> for Aggregation<'ast> {
    fn accept(&'ast self, visitor: &mut V) {
        visitor.visit_aggregation(self);
    }
}

impl<'ast, V: Visitor<'ast>> Visitable<'ast, V> for Sort<'ast> {
    fn accept(&'ast self, visitor: &mut V) {
        visitor.visit_sort(self);
    }
}
