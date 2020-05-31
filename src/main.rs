#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate lalrpop_util;

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
    use serde::{Deserialize, Serialize};
    use serde_json::{Result, Value};
    use serde_json::json;
    use regex::Regex;
    use std::collections::HashMap;
    
    lalrpop_mod!(pub search);

    #[test]
    fn sketch() {

        // ingress protocol.kitchen !feedme !"GET /assets"
        // | where stream != "stderr"
        // | where kubernetes.namespace_name = "protocol-kitchen"
        // | parse log with /"([^ ]+) ([^ ]+) HTTP\/1.1" ([\d]{3})/ as verb, path, response_code
        // | where response_code = "200"
        // | count by verb, path # multi-count maybe not mvp
        // | sort by _count

        let group_keys = vec!["verb", "path"];

        let file = File::open("data.sample.log").expect("failed to file");
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

            
            // .map(f: F);
        let result: String = filtered.iter().map(|json| serde_json::to_string(&json).unwrap()).collect();
        
        println!("{}", result);
    }

    #[test]
    fn lalrpop() {
        assert!(search::SearchParser::new().parse("ingress").is_ok());
        assert!(search::SearchParser::new().parse("protocol.kitchen").is_ok());
        assert!(search::SearchParser::new().parse("ingress protocol.kitchen").is_ok());
        assert!(search::SearchParser::new().parse(r#"ingress protocol.kitchen !feedme !"GET /assets"#).is_ok());
        // assert!(search::SearchParser::new().parse(r#"
        // ingress protocol.kitchen !feedme !"GET /assets"
        // | where stream != "stderr"
        // | where kubernetes.namespace_name = "protocol-kitchen"
        // | parse log with /"([^ ]+) ([^ ]+) HTTP\/1.1" ([\d]{3})/ as verb, path, response_code
        // | where response_code = "200"
        // | count by verb, path # multi-count maybe not mvp
        // | sort by _count"#).is_ok());
        // assert!(search::SearchParser::new().parse(r#"
        // ingress protocol.kitchen !feedme !"GET /assets"
        // | where stream != "stderr"
        // | where kubernetes.namespace_name = "protocol-kitchen"
        // | parse log with /"([^ ]+) ([^ ]+) HTTP\/1.1" ([\d]{3})/ as verb, path, response_code
        // | where response_code = "200"
        // | count by verb, path # multi-count maybe not mvp
        // | sort by _count"#).is_ok());
        // assert!(search::SearchParser::new().parse(r#"
        // ingress protocol.kitchen !feedme !"GET /assets"
        // | where stream != "stderr"
        // | where kubernetes.namespace_name = "protocol-kitchen"
        // | parse log with /"([^ ]+) ([^ ]+) HTTP\/1.1" ([\d]{3})/ as verb, path, response_code
        // | where response_code = "200"
        // | count by verb, path # multi-count maybe not mvp
        // | sort by _count"#).is_ok());
        // assert!(search::SearchParser::new().parse(r#"
        // ingress protocol.kitchen !feedme !"GET /assets"
        // | where stream != "stderr"
        // | where kubernetes.namespace_name = "protocol-kitchen"
        // | parse log with /"([^ ]+) ([^ ]+) HTTP\/1.1" ([\d]{3})/ as verb, path, response_code
        // | where response_code = "200"
        // | count by verb, path # multi-count maybe not mvp
        // | sort by _count"#).is_ok());
    }
}

