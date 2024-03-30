use anyhow::Ok;
mod defaults {
    pub(super) const PORT: u16 = 6379;
}
#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) port: u16,
}
impl Args {
    // TODO: Handle errors properly
    pub(crate) fn parse() -> anyhow::Result<Args> {
        // NOTE: The first argument is our binary's path
        let mut args = ::std::env::args().skip(1);
        let port = match args.next().as_deref() {
            Some("--port") => {
                let port = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Expected a port number!"))?
                    .parse()?;
                Ok(port)
            }
            Some(flag) => Err(anyhow::anyhow!("Unknown flag \"{flag}\"!")),
            None => Ok(defaults::PORT),
        }?;
        Ok(Args { port });
1
    }
}