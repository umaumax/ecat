# ecat

enhanced cat

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

color code: [https://upload\.wikimedia\.org/wikipedia/commons/1/15/Xterm\_256color\_chart\.svg]( https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg )

## TODO
[ ] create test code
