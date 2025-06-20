mod config;
mod parser;
pub(crate) mod sources;
mod webserver;

const HELP: &str = "
    aggregator

    USAGE:
    aggregator [OPTIONS]

    FLAGS:
    -h, --help            Prints help information

    OPTIONS:
    --port NUMBER         Sets the port for the webserver
    --addr SOCKETADDR     Sets the socket address for the webserver (--addr takes priority over --port in case of overlap)
";

#[tokio::main]
async fn main() {
    let mut arguments = pico_args::Arguments::from_env();
    if arguments.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let app_config = parser::parse_arguments(arguments).unwrap();
    webserver::run_ws(app_config).await;
}
