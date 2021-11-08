# nvhosts

Generate nginx vhosts given a configuration file. Made to work with [mlcdf/cc-cellar-proxy](https://github.com/mlcdf/cc-cellar-proxy).

## Usage

```
Usage: nvhosts [-c <config>] [--example] [-v] [-V]

Generate nginx vhosts from a configuration file

Options:
  -c, --config      path to config file to use; defaults to nvhosts.toml
  --example         show an example config
  -v, --verbose     print verbose output
  -V, --version     show the version
  --help            display usage information
```