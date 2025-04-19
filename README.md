# solana-metrics-duplicator

It works as a mirror proxy.
Launch it and direct the metrics from the Solana validator into it.
Upon receiving a packet with metrics, it will send it to your server (in any case)
and might send a copy to the Solana Labs metrics server (only if the packet looks
like it's metrics from a Solana validator; the database name is checked).

```
Sending logic:
is_solana_db     + - + - + -
is_solana_url    + + - - + +
is_mirror_url    + + + + - -
send_to_solana   S - - - S -
send_to_mirror   A S S S - -

The sync response will be returned to the client
```

```
Usage: solana-metrics-duplicator [OPTIONS] --bind-address <HOST:PORT>

At least one metrics url must be provided

Options:
      --bind-address <HOST:PORT>  
      --mirror-metrics-url <URL>  
      --solana-metrics-url <URL>  
  -h, --help                      Print help
  -V, --version                   Print version
```
