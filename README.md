# solana-project-duplicator

It works as a mirror proxy.
Launch it and direct the metrics from the Solana validator into it. 
Upon receiving a packet with metrics, it will send it to your server (in any case) 
and might send a copy to the Solana Labs metrics server (only if the packet looks 
like it's metrics from a Solana validator; the database name is checked).

```
solana-metrics-duplicator --help
Usage: solana-metrics-duplicator [OPTIONS] --bind-address <HOST:PORT> --mirror-metrics-url <URL>

Options:
      --bind-address <HOST:PORT>  
      --mirror-metrics-url <URL>  
      --solana-metrics-url <URL>  [default: https://metrics.solana.com:8086]
  -h, --help                      Print help
  -V, --version                   Print version
```
