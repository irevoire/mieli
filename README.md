# Mieli

## Usage

```
A stupid wrapper around meilisearch

USAGE:
    mieli [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --addr <addr>            The server address in the format of ip_addr:port (ex: http://0.0.0.0:7700) [env:
                                 MEILI_ADDR=]  [default: http://localhost:7700]
    -i, --index <index>          The name of the index [default: mieli]
        --interval <interval>    Interval between each status check (in milliseconds) [default: 500]

SUBCOMMANDS:
    add         Add documents with the `post` verb You can pipe your documents in the command
    delete      Delete documents. If no argument are specified all documents are deleted
    dump        Create a dump or get the status of a dump
    get         Get one document. If no argument are specified it returns all documents
    health      Do an healthcheck
    help        Prints this message or the help of the given subcommand(s)
    search      Do a search. You must pipe your parameter in the command as a json
    settings    Update the settings. You must pipe your parameter in the command as a json
    stats       Return the stats about the indexes
    status      Return the status of an update
    update      Replace documents with the `put` verb You can pipe your documents in the command
    version     Return the version of the running meilisearch instance
```

## Installation

```
git clone https://irevoire/mieli
cargo install path mieli
```

