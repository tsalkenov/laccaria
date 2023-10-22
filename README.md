# laccaria
Laccaria is daemon based process manager with cli. 
### Usage
Simply launch daemon and use it
```
$ laccaria-daemon
$ laccaria --help
Usage: laccaria <COMMAND>
Commands:
  start    Start process by giving it name and command
  kill     Kill process
  delete   Permanently remove any state conne related to process
  list     List all processes
  restart  Restart saved process
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

$ laccaria start --auto-restart "sleep" "sleep 10"
INFO  laccaria::commands::start > Starting process sleep
INFO  laccaria::commands::start > Process started
```
