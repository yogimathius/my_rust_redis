use anyhow::Ok;
mod defaults {
    pub(super) const PORT: u16 = 6379;
    pub(super) const ROLE: crate::Role = crate::Role::Master;
}
#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) port: u16,
    pub(crate) role: crate::Role,
}
impl Args {
    // TODO: Handle errors properly
    pub(crate) fn parse() -> anyhow::Result<Args> {
        // NOTE: The first argument is our binary's path
        let mut args = ::std::env::args().skip(1);
        
        let mut port = defaults::PORT;
        let mut role = defaults::ROLE;

        // let port = match args.next().as_deref() {
        //     Some("--port") => {
        //         let port = args
        //             .next()
        //             .ok_or_else(|| anyhow::anyhow!("Expected a port number!"))?
        //             .parse()?;
        //         Ok(port)
        //     }
        //     Some("--replicaof")
        //     Some(flag) => Err(anyhow::anyhow!("Unknown flag \"{flag}\"!")),
        //     None => Ok(defaults::PORT),
        // }?;
        // Ok(Args { port })

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--port" => {
                    port = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Expected a port number!"))?
                        .parse()?;
                }
                "--replicaof" => {
                    let host = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Expected a host!"))?;
                    let host = match host.as_str() {
                        "localhost" => ::std::net::Ipv4Addr::LOCALHOST,
                        _ => host.parse()?,
                    };
                    let port = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Expected a port number!"))?
                        .parse()?;
                    role = crate::Role::Slave { host, port };
                }
                flag => return Err(anyhow::anyhow!("Unknown flag \"{}\"!", flag)),
            }
        }
        Ok(Args { port, role })
    }
}