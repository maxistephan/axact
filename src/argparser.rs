use clap::{
    Arg,
    ArgAction,
    ArgMatches,
    Command
};

pub struct ArgParser {
    pub host: String,
    pub port: String,
    pub show_gpu_temp: bool,
}

pub fn get_arg_parser() -> ArgParser {
    let matches: ArgMatches = Command::new("Axact")
        .version("0.2.2")
        .author("Maximilian Stephan")
        .about("A resource monitor in your browser, in Rust.")
        .arg(Arg::new("show-gpu-temp")
            .long("show-gpu-temp")
            .action(ArgAction::SetTrue)
            .default_value("false")
            .help("Show the GPU temperature aswell (works only for NVIDIA GPUs)."))
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
    let show_gpu_temp: bool = *matches.get_one::<bool>("show-gpu-temp").unwrap();

    ArgParser { host, port, show_gpu_temp }
}
