# Mieli 🐻

## Usage 🧸

```text
mieli 0.1.10
A stupid wrapper around meilisearch

USAGE:
    mieli [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --addr <addr>                The server address in the format of ip_addr:port (ex: http://0.0.0.0:7700) [env:
                                     MEILI_ADDR=]  [default: http://localhost:7700]
    -i, --index <index>              The name of the index [env: MIELI_INDEX=]  [default: mieli]
        --interval <interval>        Interval between each status check (in milliseconds) [default: 200]
    -k, --key <key>                  Your secret API key <https://docs.meilisearch.com/reference/api/keys.html#get-keys>
                                     [env: MEILI_MASTER_KEY=]
        --user-agent <user-agent>    Use a specific http User-Agent for your request

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

## Get mieli on your system 🍯

```bash
cargo install mieli
```

## Examples 🐝

### Add documents

```bash
echo '[
    { "id": 1,
    "title": "Mieli search his honey pot",
    "content": "Mieli, the fat brown bear, was looking for honey in this majestuous forest ..."
    }
]' | mieli -i cook add
```

### Search

```bash
# placeholder
mieli -i book search
# simple search
mieli -i book search honey
# complex search
echo '{ "q": "honey", "limit": 1 }' | mieli -i book search
```

By default all search are interactive. But if you pipe the result of `mieli` into another command then the search results are sent immediatly.
```bash
mieli -i book search honey | jq '.content'
```

[![asciicast](https://asciinema.org/a/439266.svg)](https://asciinema.org/a/439266)
