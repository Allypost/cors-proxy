#[derive(Debug, Clone, clap::Args)]
pub struct ServerConfig {
    /// The port on which the server will listen.
    #[arg(long, default_value = "8000", env = "PORT", value_parser = clap::value_parser!(u16).range(1..65535))]
    pub port: u16,

    /// The host on which the server will listen.
    #[arg(long, default_value = "0.0.0.0", env = "HOST")]
    pub host: String,
}
impl ServerConfig {
    pub fn addr_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
