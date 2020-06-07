#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate lalrpop_util;

mod ast;
mod visitor;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}

#[cfg(test)]
mod tests {
    extern crate serde;
    extern crate serde_json;
    extern crate regex;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;
    use serde_json::{Value};
    use serde_json::json;
    use regex::Regex;
    use std::collections::HashMap;
    use crate::ast::*;
    use crate::visitor::{Visitor, Visitable};
    
    lalrpop_mod!(pub search);

    #[test]
    fn sketch() {

        // ingress protocol.kitchen !feedme !"GET /assets"
        // | where stream != "stderr"
        // | where kubernetes.namespace_name = "protocol-kitchen"
        // | parse log with '"([^ ]+) ([^ ]+) HTTP/1.1" ([\d]{3})' as verb, path, response_code
        // | where response_code = "200"
        // | count by verb, path # multi-count maybe not mvp
        // | sort by _count

        let group_keys = vec!["verb", "path"];

        let file = File::open("mini.sample.log").expect("failed to file");
        let buf_reader = BufReader::new(file);
        let mut filtered = buf_reader.lines()
            .map(|l| l.unwrap())
            .filter(|line| line.contains("protocol.kitchen"))
            .filter(|line| !line.contains("feedme"))
            .filter(|line| !line.contains("GET /assets"))
            .filter_map(|line| serde_json::from_str(&line).ok())
            .filter(|json: &Value| json["stream"] != "stderr")
            .filter(|json| json["kubernetes"]["namespace_name"] == "protocol-kitchen")
            .map(|mut json: Value| {
                let re = Regex::new(r#""([^ ]+) ([^ ]+) HTTP/1.1" ([\d]{3})"#).unwrap();
                let cap = re.captures(json["log"].as_str().unwrap()).unwrap(); // actually here we want to filter out lines that don't match tbh
                let aliases = vec!["verb", "path", "response_code"];
                let aliases_and_value: Vec<(&&str, String)> = aliases
                    .iter().enumerate()
                    .map(|(idx, alias)| (alias, cap[idx + 1].to_owned()))
                    .collect();
                for (alias, value) in aliases_and_value {
                    json[alias] = Value::String(value);
                }
                json
            })
            .filter(|json| json["response_code"] == "200") // num would be nice but extracting from regex yields str ofc
            .fold(HashMap::new(), |mut acc, json| {
                let identity: Vec<String> = group_keys.iter().map(|k| json[k].as_str().unwrap().to_owned()).collect();
                let counter = acc.entry(identity).or_insert(0);
                *counter += 1;
                acc
            }).iter().map(|(k, v)| {
                let mut json = json!({});
                let group_keys = vec!["verb", "path"];
                for (i, key) in group_keys.iter().enumerate() {
                    json[key] = json!(k[i]);
                }
                json["_count"] = json!(v);
                json
            }).collect::<Vec<Value>>();
        
        filtered.sort_by(|a, b| serde_json::to_string(&b["_count"]).unwrap().cmp(&serde_json::to_string(&a["_count"]).unwrap()));

        let got = filtered;
        let expected = vec![
            json!({"_count":4,"path":"/index.html","verb":"GET"}),
            json!({"_count":3,"path":"/","verb":"GET"}),
            json!({"_count":2,"path":"/a-tale-of-two-clams/","verb":"GET"}),
            json!({"_count":1,"path":"/lazy-loaf-tin-bakes/","verb":"GET"})];

        assert_eq!(expected, got);
    }

    #[test]
    fn lalrpop_sketch() {
        assert!(search::SearchParser::new().parse("ingress").is_ok());
        assert!(search::SearchParser::new().parse("protocol.kitchen").is_ok());
        assert!(search::SearchParser::new().parse("ingress protocol.kitchen").is_ok());
        assert!(search::SearchParser::new().parse(r#"ingress protocol.kitchen !feedme !"GET /assets""#).is_ok());
        // println!("{}",search::SearchParser::new().parse(r#"
        // ingress protocol.kitchen !feedme !"GET /assets"
        // | where stream != "stderr""#).unwrap_err());
        assert!(search::SearchParser::new().parse(r#"
        ingress protocol.kitchen !feedme !"GET /assets"
        | where stream != "stderr""#).is_ok());
        assert!(search::SearchParser::new().parse(r#"
        ingress protocol.kitchen !feedme !"GET /assets"
        | where stream != "stderr"
        | where kubernetes.namespace_name = "protocol-kitchen"
        | parse log with '"([^ ]+) ([^ ]+) HTTP/1.1" ([\d]{3})' as verb, path, response_code
        | where response_code = "200"
        | count by verb, path
        | sort by _count"#).is_ok());
    }

    #[test]
    fn lalrpop_ast_sketch() {
        let (search_terms, transforms, sort): Search = *search::SearchParser::new().parse(r#"
        ingress protocol.kitchen !feedme !"GET /assets"
        | where stream != "stderr"
        | where kubernetes.namespace_name = "protocol-kitchen"
        | parse log with '"([^ ]+) ([^ ]+) HTTP/1.1" ([\d]{3})' as verb, path, response_code
        | where response_code = "200"
        | count by verb, path
        | sort by _count"#).unwrap();
        assert_eq!(vec![
            SearchTerm::Include("ingress"),
            SearchTerm::Include("protocol.kitchen"),
            SearchTerm::Exclude("feedme"),
            SearchTerm::Exclude(r#"GET /assets"#), // FIXME: needs unquoting but might be better done by the ast analyser rathen than in-parsing (replace changes types from &'input str to String with temp ownership)
            ], search_terms);
        assert_eq!(vec![
            Transform::Filter { field: "stream", comparison: Comparison::Ne, value: r#"stderr"#},
            Transform::Filter { field: "kubernetes.namespace_name", comparison: Comparison::Eq, value: r#"protocol-kitchen"#},
            Transform::Parse { field: "log", parser: r#""([^ ]+) ([^ ]+) HTTP/1.1" ([\d]{3})"#, bindings: vec!["verb", "path", "response_code"]},
            Transform::Filter { field: "response_code", comparison: Comparison::Eq, value: r#"200"#},
            Transform::Aggregate(Aggregation::Count(vec!["verb", "path"])),
        ], transforms);
        assert_eq!(Some(Sort::Desc(vec!["_count"])), sort);
    }


    struct TestVisitor {
        include_terms: usize,
        exclude_terms: usize,
        any: usize,
        filters_equal: usize,
        filters_not_equal: usize,
        filters_match: usize,
        parses: usize,
        // captured_fields: usize,
        bound_fields: usize,
        aggregations: usize,
        count_fields: usize,
        sort_fields: usize,
    }

    impl Visitor<'_> for TestVisitor {
        fn visit_search(&mut self, _search: &Search) {}
        fn visit_search_term(&mut self, search_term: &SearchTerm) {
            match search_term {
                SearchTerm::Include(_) => self.include_terms += 1,
                SearchTerm::Exclude(_) => self.exclude_terms += 1,
                SearchTerm::Any() => self.any += 1,
            }
        }
        fn visit_transform(&mut self, transform: &Transform) {
            match transform {
                Transform::Aggregate(_) => self.aggregations += 1,
                Transform::Filter {field: _, comparison, value: _} => match comparison {
                    Comparison::Eq => self.filters_equal += 1,
                    Comparison::Ne => self.filters_not_equal += 1,
                    Comparison::Match => self.filters_match += 1,
                },
                Transform::Parse {field: _, parser: _, bindings} => {
                    self.parses += 1;
                    self.bound_fields += bindings.len();
                },
            }
        }
        fn visit_aggregation(&mut self, aggregation: &Aggregation) {
            match aggregation {
                Aggregation::Count(fields) => self.count_fields += fields.len(),
            }
        }
        fn visit_sort(&mut self, sort: &Sort) {
            match sort {
                Sort::Asc(fields) => self.sort_fields += fields.len(),
                Sort::Desc(fields) => self.sort_fields += fields.len(),
            }
        }
    }

    #[test]
    fn lalrpop_ast_visitor_sketch() {
        let search: Search = *search::SearchParser::new().parse(r#"
        ingress protocol.kitchen !feedme !"GET /assets"
        | where stream != "stderr"
        | where kubernetes.namespace_name = "protocol-kitchen"
        | parse log with '"([^ ]+) ([^ ]+) HTTP/1.1" ([\d]{3})' as verb, path, response_code
        | where response_code = "200"
        | count by verb, path
        | sort by _count"#).unwrap();
        let mut test_visitor = TestVisitor {
            include_terms: 0,
            exclude_terms: 0,
            any: 0,
            filters_equal: 0,
            filters_not_equal: 0,
            filters_match: 0,
            parses: 0,
            bound_fields: 0,
            aggregations: 0,
            count_fields: 0,
            sort_fields: 0,
        };
        search.accept(&mut test_visitor);
        assert_eq!(2 ,test_visitor.include_terms);
        assert_eq!(2 ,test_visitor.exclude_terms);
        assert_eq!(0 ,test_visitor.any);
        assert_eq!(2 ,test_visitor.filters_equal);
        assert_eq!(1 ,test_visitor.filters_not_equal);
        assert_eq!(0 ,test_visitor.filters_match);
        assert_eq!(1 ,test_visitor.parses);
        assert_eq!(3 ,test_visitor.bound_fields);
        assert_eq!(1 ,test_visitor.aggregations);
        assert_eq!(2 ,test_visitor.count_fields);
        assert_eq!(1 ,test_visitor.sort_fields);
    }

    #[test]
    fn sketch_type_breakdown() {
        let group_keys = vec!["verb", "path"];

        let file = File::open("mini.sample.log").expect("failed to file");
        let buf_reader = BufReader::new(file);

        let filtered1: &mut dyn Iterator<Item = std::result::Result<std::string::String, std::io::Error>> = &mut buf_reader.lines();
        let filtered2: &mut dyn Iterator<Item = String> = &mut filtered1.map(|l| l.unwrap());
        // let filtered3: &mut dyn Iterator<Item = String> = &mut filtered2.filter(|line| line.contains("ingress")); LOL THIS FILTERS TOO MUCH, MISSED IN ORIGINAL TEST!
        let filtered4: &mut dyn Iterator<Item = String> = &mut filtered2.filter(|line| line.contains("protocol.kitchen"));
        let filtered5: &mut dyn Iterator<Item = String> = &mut filtered4.filter(|line| !line.contains("feedme"));
        let filtered6: &mut dyn Iterator<Item = String> = &mut filtered5.filter(|line| !line.contains("GET /assets"));
        let filtered7: &mut dyn Iterator<Item = Value> = &mut filtered6.filter_map(|line| serde_json::from_str(&line).ok());
        let filtered8: &mut dyn Iterator<Item = Value> = &mut filtered7.filter(|json: &Value| json["stream"] != "stderr");
        let filtered9: &mut dyn Iterator<Item = Value> = &mut filtered8.filter(|json| json["kubernetes"]["namespace_name"] == "protocol-kitchen");
        let filtered10: &mut dyn Iterator<Item = Value> = &mut filtered9.map(|mut json: Value| {
            let re = Regex::new(r#""([^ ]+) ([^ ]+) HTTP/1.1" ([\d]{3})"#).unwrap();
            let cap = re.captures(json["log"].as_str().unwrap()).unwrap(); // actually here we want to filter out lines that don't match tbh
            let aliases = vec!["verb", "path", "response_code"];
            let aliases_and_value: Vec<(&&str, String)> = aliases
                .iter().enumerate()
                .map(|(idx, alias)| (alias, cap[idx + 1].to_owned()))
                .collect();
            for (alias, value) in aliases_and_value {
                json[alias] = Value::String(value);
            }
            json
        });
        let filtered11: &mut dyn Iterator<Item = Value> = &mut filtered10.filter(|json| json["response_code"] == "200");
        let filtered12: HashMap<Vec<String>, usize> = filtered11.fold(HashMap::new(), |mut acc, json| {
            let identity: Vec<String> = group_keys.iter().map(|k| json[k].as_str().unwrap().to_owned()).collect();
            let counter = acc.entry(identity).or_insert(0);
            *counter += 1;
            acc
        });
        let filtered13: &mut dyn Iterator<Item = Value> = &mut filtered12.iter().map(|(k, v)| {
            let mut json = json!({});
            let group_keys = vec!["verb", "path"];
            for (i, key) in group_keys.iter().enumerate() {
                json[key] = json!(k[i]);
            }
            json["_count"] = json!(v);
            json
        });
        let mut filtered = filtered13.collect::<Vec<Value>>();
        
        filtered.sort_by(|a, b| serde_json::to_string(&b["_count"]).unwrap().cmp(&serde_json::to_string(&a["_count"]).unwrap()));

        let got = filtered;
        let expected = vec![
            json!({"_count":4,"path":"/index.html","verb":"GET"}),
            json!({"_count":3,"path":"/","verb":"GET"}),
            json!({"_count":2,"path":"/a-tale-of-two-clams/","verb":"GET"}),
            json!({"_count":1,"path":"/lazy-loaf-tin-bakes/","verb":"GET"})];

        assert_eq!(expected, got);
    }


}

