use std::path::PathBuf;

use pingora::server::configuration::Opt as PingoraOpt;

#[derive(Debug, Clone, clap::Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct PingoraConfig {
    /// Whether this server should try to upgrade from an running old server
    #[clap(short, long, env = "CORS_PROXY_SHOULD_UPGRADE")]
    pub upgrade: bool,

    /// Whether should run this server in the background
    #[clap(short, long, env = "CORS_PROXY_RUN_AS_DAEMON")]
    pub daemon: bool,

    /// Test the configuration and exit
    ///
    /// This flag is useful for upgrading service where the user wants to make sure the new
    /// service can start before shutting down the old server process.
    #[clap(short, long, env = "CORS_PROXY_TEST_CONFIG")]
    pub test_config: bool,

    /// The path to the Pingora server configuration file.
    /// Should be a YAML file.
    ///
    /// Reference: https://docs.rs/pingora/0.1.0/pingora/server/configuration/struct.ServerConf.html
    #[clap(short, long, value_name = "FILE_PATH", env = "CORS_PROXY_CONFIG_PATH")]
    pub config_file: Option<PathBuf>,

    /// Not actually used. This flag is there so that the server is not upset seeing this flag
    /// passed from `cargo test` sometimes
    #[clap(long, hide = true)]
    pub nocapture: bool,
}
impl PingoraConfig {
    pub fn as_pingora_opt(&self) -> PingoraOpt {
        PingoraOpt {
            upgrade: self.upgrade,
            daemon: self.daemon,
            nocapture: self.nocapture,
            test: self.test_config,
            conf: self
                .config_file
                .clone()
                .map(|x| x.as_os_str().to_string_lossy().to_string()),
        }
    }
}

impl From<PingoraConfig> for PingoraOpt {
    fn from(pingora: PingoraConfig) -> Self {
        pingora.as_pingora_opt()
    }
}
