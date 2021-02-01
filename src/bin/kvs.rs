use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("kvs")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("key").index(1).required(true))
                .arg(Arg::with_name("value").index(2).required(true)),
        )
        .subcommand(SubCommand::with_name("get").arg(Arg::with_name("key").required(true)))
        .subcommand(SubCommand::with_name("rm").arg(Arg::with_name("key").required(true)))
        .get_matches();

    match matches.subcommand() {
        ("set", Some(_set_matches)) => {
            eprintln!("unimplemented");
        }
        ("get", Some(_get_matches)) => {
            eprintln!("unimplemented");
        }
        ("rm", Some(_rm_matches)) => {
            eprintln!("unimplemented");
        }
        _ => unreachable!(),
    }

    std::process::exit(1);
}
