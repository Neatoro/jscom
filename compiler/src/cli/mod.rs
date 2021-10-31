use clap::{Arg, App};

pub struct Options {
    pub input: String
}

pub fn parse_opts() -> Options {
    let matches = App::new("jscom")
        .version("1.0")
        .about("Small LLVM compiler for JS")
        .arg(
            Arg::from_usage("<INPUT>")
                .required(true)
                .index(1)
        )
        .get_matches();

    let mut input: String = String::new();
    if let Some(i) = matches.value_of("INPUT") {
        input = String::from(i);
    }

    Options {
        input
    }
}
