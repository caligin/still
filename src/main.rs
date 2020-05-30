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
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    #[test]
    fn sketch() {
        let file = File::open("data.sample.log").expect("failed to file");
        let buf_reader = BufReader::new(file);
        let filtered = buf_reader.lines()
            .map(|l| l.unwrap())
            .filter(|line| line.contains("ingress"));
        let result: String = filtered.collect();
        
        println!("AAAAAAAAAAAAAAAAAAS {}", result);
    }
}
