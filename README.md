# PgRestore Web

Web interface to local pg_restore command line utility.

## Usage

Start pgrestore-web with default configuration:

```bash
./pgrestore-web
```

Optional arguments:

* `-a` (`--address`) ADDR: Address to listen on, default value - localhost;
* `-p` (`--port`) PORT: Port to listen on, default value - 8080;
* `-c` (`--config`) PATH: Path to configuration file, default value - config.yaml;
* `-h` (`--help`): Show help and exit.

## Configuration Example

Simple configuration example:

```yaml
---
max_jobs: 10
restore_jobs: 8

commands:
  createdb_path: /usr/bin/createdb
  dropdb_path: /usr/bin/dropdb
  pgrestore_path: /usr/bin/pg_restore
  psql_path: /usr/bin/psql

destinations:
  - host: localhost
    port: 5432
    role: user_one
    password: pass_one

  - host: localhost
    port: 5432
    role: user_two
    password: pass_two
```
