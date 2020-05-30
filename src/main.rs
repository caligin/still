#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

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
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::Result;

    #[test]
    fn sketch() {

        // ingress protocol.kitchen !feedme !"GET /assets"
        // | where stream != "stderr"
        // | where kubernetes.namespace_name = "protocol-kitchen"
        // | parse log with /"([^ ]+) ([^ ]+) HTTP\/1.1" ([\d]{3})/ as verb, path, response_code
        // | where response_code = 200
        // | count by verb, path # multi-count maybe not mvp
        // | sort by _count

        let file = File::open("data.sample.log").expect("failed to file");
        let buf_reader = BufReader::new(file);
        let filtered = buf_reader.lines()
            .map(|l| l.unwrap())
            .filter(|line| line.contains("protocol.kitchen"))
            .filter(|line| !line.contains("feedme"))
            .filter(|line| !line.contains("GET /assets"))
            .filter_map(|line| serde_json::from_str(line).ok())
            .filter(|json| json["stream"] != "stderr");
            
            // .map(f: F);
        let result: String = filtered.collect();
        
        println!("{}", result);
    }
}

// let p: Person = serde_json::from_str(data)?