use clap::{
    Arg,
    ArgAction,
    ArgMatches,
    Command
};

pub struct ArgParser {
    pub host: String,
    pub port: String,
    pub use_liquidctl: bool,
}

pub fn get_arg_parser() -> ArgParser {
    let matches: ArgMatches = Command::new("Axact")
        .version("0.2.1")
        .author("Maximilian Stephan")
        .about("A resource monitor in your browser, in Rust.")
        .arg(Arg::new("use-liquidctl")
            .short('l')
            .long("use-liquidctl")
            .default_value("false")
            .action(ArgAction::SetTrue)
            .help("Use liquidctl to setup a fan curve based off of you gpu and cpu temperature."))
        .arg(Arg::new("host")
            .long("host")
            .default_value("0.0.0.0")
            .help("IP address for the Web Server to use."))
        .arg(Arg::new("port")
            .short('p')
            .long("port")
            .default_value("7032")
            .help("Port for the Web Server to use."))
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap().to_owned();
    let port = matches.get_one::<String>("port").unwrap().to_owned();
    let use_liquidctl: bool = *matches.get_one::<bool>("use-liquidctl").unwrap();

    ArgParser { host, port, use_liquidctl }
}
