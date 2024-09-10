use autopower_shared::{
    logging::Logger,
    pipe::{Pipe, Server, PIPE_NAME},
    proxy_command::ProxyCommand,
    stream::Write,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const LOGGER: Logger = Logger::new("proxy", "autopower");

pub struct Proxy {
    pipe: Pipe<Server, Write>,
}

impl Proxy {
    pub fn create() -> Result<Self> {
        LOGGER.debug("Creating pipe...");
        let pipe = Pipe::create_server(PIPE_NAME)?;
        LOGGER.debug("Created pipe, waiting for connection...");
        pipe.connect()?;
        Ok(Proxy { pipe })
    }
    pub fn send_command(&mut self, command: ProxyCommand) -> Result<()> {
        LOGGER.debug(format!("Sent command:\n{:?}", command));
        self.pipe.write_as(command)?;
        Ok(())
    }

    pub fn terminate(&mut self) -> Result<()> {
        LOGGER.debug("Terminating proxy...");
        self.pipe.close()
    }
}
