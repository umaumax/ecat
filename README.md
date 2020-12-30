# ecat

enhanced cat

colorize terminal output tool

## how to install
``` bash
cargo install --git https://github.com/umaumax/ecat
```

## NOTE
* no support to multi-line match

## how to run
e.g.
``` bash
ifconfig | ecat
df | ecat
```

``` bash
$ ecat -h
ecat 0.0.1
enhanced cat command by rust

USAGE:
    ecat [FLAGS] [OPTIONS] [files]...

FLAGS:
    -h, --help       Prints help information
    -n               number the output lines, starting at 1
    -V, --version    Prints version information

OPTIONS:
        --color <WHEN>       use markers to highlight the mathing strings; WHEN is [always], [never], or [auto]
        --config <STRING>    set config filepath
    -C, --context <NUM>      print NUM lines of output context
        --line <NUM>         print taeget line of output context

ARGS:
    <files>...    Sets the input file to use (default is /dev/stdin) [default: -]
```

## config file
1. `./config.yaml`
2. `~/.config/ecat/config.yaml`

e.g.
``` yaml
- name: rust_macro
  patterns: ["print!", "write!"]
  color: "#ff875f"

- name: cpp_comment
  patterns: ["//.*", "/\\*.*\\*/"]
  color: "241"
```

* color code sheet: [https://upload\.wikimedia\.org/wikipedia/commons/1/15/Xterm\_256color\_chart\.svg]( https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg )

## TODO
* [ ] create test code
* [ ] refactor code
* [ ] attach default config.yaml
