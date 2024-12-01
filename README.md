backend
=======

[<img alt="github" src="https://img.shields.io/badge/github-BashSdo/real--estate--agency--backend-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/BashSdo/real-estate-agency-backend)

Backend application for the real estate agency project.

# Building

To build a release Docker image run the following command:
```sh
just image_build
```

# Configuration

There is multiple ways to configure the application:
- using a configuration file;
- using environment variables.

## Configuration file

Configuration file can be specified by providing `-c`/`--config` argument with the path to the file (`./config.toml` will be used by default).

Example of the configuration file can be found in the [config.toml](config.toml). 

## Environment variables

Environment variables follows the following pattern: `CONF.<SECTION>.<SECTION>.<KEY>`.

For example the following configuration:
```toml
[server]
host = "0.0.0.0"
port = 8080
```

can be specified via the following environment variables:
```sh
CONF.SERVER.HOST="0.0.0.0"
CONF.SERVER.PORT=8080
```

Note, that not all shells support `.` in the environment variables names, in this case you can use `env` command to set the environment variables:
```sh
env CONF_SERVER_HOST="0.0.0.0" CONF_SERVER_PORT=8080 just run
```

Before sending a pull request, please make sure you have read the [contributing guidelines](CONTRIBUTING.md).
