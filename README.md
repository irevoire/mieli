# Mieli 🐻

## Usage 🧸

```text
A stupid wrapper around meilisearch

Usage: mieli [OPTIONS] <COMMAND>

Commands:
  self       Modify the `mieli` installation
  documents  Manipulate documents, add `--help` to see all the subcommands
  dump       Create a dump
  tasks      Get information about the task of an index
  health     Do an healthcheck
  version    Return the version of the running meilisearch instance
  stats      Return the stats about the indexes
  search     Do a search. You can pipe your parameter in the command as a json. Or you can specify directly what you want to search in the arguments
  settings   Get or update the settings. You can pipe your settings in the command
  index      Manipulate indexes, add `--help` to see all the subcommands
  key        Get the keys
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...                     
  -a, --addr <ADDR>                    The server address in the format of ip_addr:port (ex: http://0.0.0.0:7700) [env: MEILI_ADDR=] [default: http://localhost:7700]
      --async                          The command will exit immediatly after executing
  -i, --index <INDEX>                  The name of the index [env: MIELI_INDEX=] [default: mieli]
  -k, --key <KEY>                      Your secret API key <https://docs.meilisearch.com/reference/api/keys.html#get-keys> [env: MEILI_MASTER_KEY=]
      --user-agent <USER_AGENT>        Use a specific http User-Agent for your request [default: mieli/0.28.2]
      --custom-header <CUSTOM_HEADER>  Use a specific http header for your request. Eg. `mieli search --custom-header "x-meilisearch-client: turbo-doggo/42.9000"`
      --interval <INTERVAL>            Interval between each status check (in milliseconds) [default: 200]
  -h, --help                           Print help```

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
]' | mieli -i cook documents add
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
