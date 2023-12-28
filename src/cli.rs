use {
    clap::Parser,
    hyper::Uri,
    log::*,
    std::{net::SocketAddrV4, str::FromStr},
};

#[derive(Debug, Parser)]
#[command(author, version, about, verbatim_doc_comment)]
struct ClapArgs {
    #[arg(long, value_name = "HOST:PORT")]
    bind_address: String,

    #[arg(long, value_name = "URL")]
    mirror_metrics_url: String,

    #[arg(
        long,
        value_name = "URL",
        default_value = "https://metrics.solana.com:8086"
    )]
    solana_metrics_url: String,
}

#[derive(Debug)]
pub struct Args {
    pub bind_addr: SocketAddrV4,
    pub solana_metrics_url: String,
    pub mirror_metrics_url: String,
}

impl Args {
    fn from_clap_args(clap_args: ClapArgs) -> Self {
        let sanitaze_url = |url| {
            let mut url = Uri::from_str(url).unwrap().to_string();
            if url.chars().last() == Some('/') {
                url.pop();
            }
            url
        };
        Self {
            bind_addr: SocketAddrV4::from_str(clap_args.bind_address.as_str()).unwrap(),
            solana_metrics_url: sanitaze_url(clap_args.solana_metrics_url.as_str()),
            mirror_metrics_url: sanitaze_url(clap_args.mirror_metrics_url.as_str()),
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
