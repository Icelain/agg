pub struct Config {
    pub webserver_address: Option<std::net::SocketAddr>,
    pub webserver_port: Option<u32>,
    pub openai_key: String,
}
