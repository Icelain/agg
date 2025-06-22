use dotenv::dotenv;

use crate::config;

pub fn parse_arguments_and_env_vars(
    mut args: pico_args::Arguments,
) -> Result<config::Config, anyhow::Error> {
    let port_opt = args.value_from_str("--port");
    let address_opt = args.value_from_str("--addr");

    if port_opt.is_err() && address_opt.is_err() {
        return Err(anyhow::anyhow!(
            "Neither port or socket address provided for the webserver"
        ));
    }

    dotenv();

    Ok(config::Config {
        webserver_address: address_opt.ok(),
        webserver_port: port_opt.ok(),
        openai_key: std::env::var("OPENAI_API_KEY")?,
    })
}
