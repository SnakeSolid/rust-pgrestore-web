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

## Dependencies

This utility internally uses `createdb`, `dropdb` and `pg_restore` command line tools from `postgresql-client`.

## Configuration Example

Simple configuration example:

```yaml
---
max_jobs: 10 # maximal number of jobs to store results
indexes_path: "indexes.csv" # optional path to CSV file with table indexes
joblogs_path: "logs" # directory to store restore jobs output
restore_jobs: 8 # number of jobs for pg_restore command

templates: # template settings
  full: "template0" # optional template for restoring full database backup
  partial: "template0" # optional template for restoring partial backup (schema's or table)

search_config: # backup search configuration
  interval: 21600 # scanning interval in seconds
  recursion_limit: 8 # limit directory level recursion (default 5)
  directories: # directories to scan (if empty - disable directory scanner)
    - "/mnt/tape1/backups"
    - "/mnt/tape2/backups"
  extensions: # backup file extensions (if empty - disable directory scanner)
    - "dump"
    - "backup"

http_server: # HTTP server settings
  cors: # Cross-origin resource sharing setting (default: cors disabled)
    type: AllowAny # allow all requests

http_client: # HTTP client settings
  download_directory: /tmp # directory to store temporary downloaded files
  root_certificates: [] # list root certificates in PEM format, if MITM proxy used
  accept_invalid_hostnames: false # accept invalid SSL certificates (default: false)
  accept_invalid_certs: false # accept invalid SSL host names (default: false)

commands: # paths to PostgreSQL command line utilities
  createdb_path: /usr/bin/createdb
  dropdb_path: /usr/bin/dropdb
  pgrestore_path: /usr/bin/pg_restore

destinations: # list of PostgreSQL servers to restore database
  - host: localhost # host name
    port: 5432 # port
    role: user_one # user name with create database / drop database access
    password: pass_one # user password

  - host: localhost
    port: 5432
    role: user_two
    password: pass_two
```

## Indexes File

Parameter `indexes_path` in configuration contains path to indexes file. This file used to match table in backup with
corresponding indexes. Indexes will be read for every restore operation.

File must contain three columns separated by commas without any headers:

1. schema name;
2. table name;
3. index name.

Schema name is the same for table and index. Example of file content:

```csv
public,table_first,pk_table_first_id
public,table_first,nx_table_first_name
public,table_second,pk_table_second_id
public,table_second,nx_table_second_category
```
