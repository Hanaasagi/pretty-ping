# pretty-ping

A ping command with colorful output.

[![asciicast](https://asciinema.org/a/428082.svg)](https://asciinema.org/a/428082)


## Installation

```Bash
cargo install --git https://github.com/Hanaasagi/pretty-ping
```

**This binary requires the ability to create raw sockets. Run as root or explicitly set `sudo setcap cap_net_raw=eip $(which pretty-ping)`.**
## Usage

```
USAGE:
    pretty-ping [OPTIONS] <HOSTNAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --count <COUNT>          stop after <count> replies [default: 0]
    -i, --interval <INTERVAL>    millisecond between sending each packet [default: 1000]
    -s, --packetsize <SIZE>      specify the number of data bytes to be sent.  The default is 56, which translates into
                                 64 ICMP data bytes when combined with the 8 bytes of ICMP header data. [default: 56]
    -t, --timeout <TIMEOUT>      millisecond to wait for response [default: 1000]

ARGS:
    <HOSTNAME>    domain name or ip address
```

## License

BSD 3-Clause License Copyright (c) 2021, 秋葉.
