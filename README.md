# :eyes: supervise

a simple command line supervisor tool.

```
Supervise command execution.

USAGE:
    supervise [OPTIONS] [COMMAND]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --count <count>          maximum number of executions
    -i, --interval <interval>    execution interval (sec) [default: 0.1]

ARGS:
    <COMMAND>...    command and options
```

## example

```shell
$ supervice -c 2 -i 2 -- echo abc
```
