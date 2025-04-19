use {
    clap::Parser,
    hyper::Uri,
    log::*,
    std::{net::SocketAddrV4, str::FromStr},
};

/// Sending logic:
/// is_solana_db     + - + - + -
/// is_solana_url    + + - - + +
/// is_mirror_url    + + + + - -
/// send_to_solana   S - - - S -
/// send_to_mirror   A S S S - -
/// The sync response will be returned to the client
#[derive(Debug, Parser)]
#[command(author, version, about, verbatim_doc_comment)]
struct ClapArgs {
    #[arg(long, value_name = "HOST:PORT")]
    bind_address: String,

    #[arg(long, value_name = "URL")]
    mirror_metrics_url: Option<String>,

    #[arg(long, value_name = "URL")]
    solana_metrics_url: Option<String>,
}

#[derive(Debug)]
pub struct Args {
    pub bind_addr: SocketAddrV4,
    pub solana_metrics_url: Option<String>,
    pub mirror_metrics_url: Option<String>,
}

impl Args {
    fn from_clap_args(clap_args: ClapArgs) -> Self {
        let sanitize_url = |url: &Option<String>| {
            url.as_ref().map(|url| {
                let mut url = Uri::from_str(url).unwrap().to_string();
                if url.ends_with('/') {
                    url.pop();
                }
                url
            })
        };
        if clap_args.solana_metrics_url.is_none() && clap_args.mirror_metrics_url.is_none() {
            error!("At least one metrics url must be provided");
            std::process::exit(1);
        }
        Self {
            bind_addr: SocketAddrV4::from_str(clap_args.bind_address.as_str()).unwrap(),
            solana_metrics_url: sanitize_url(&clap_args.solana_metrics_url),
            mirror_metrics_url: sanitize_url(&clap_args.mirror_metrics_url),
        }
    }
}

impl Args {
    pub fn parse() -> Self {
        let clap_args = <ClapArgs as clap::Parser>::parse();
        debug!("clap_args:\n{:?}", clap_args);
        let args = Self::from_clap_args(clap_args);
        info!("Args:\n{:#?}", args);
        args
    }
}
