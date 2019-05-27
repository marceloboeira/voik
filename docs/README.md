<p align="center">
  <img src="https://github.com/marceloboeira/voik/blob/master/docs/logos/github.png?raw=true" width="400">
  <p align="center">An experiment on persistent data-streaming<p>
  <p align="center">
    <img src="https://travis-ci.org/14-bits/voik.svg?branch=master">
  </p>
</p>

## Status

Currently, working in the foundation of the **storage layer**.

To know more about component internals, performance and references, please check the [ARCHITECTURE.md](https://github.com/14-bits/voik/blob/master/docs/ARCHITECTURE.md).

## Project Goals

* Learn and have fun™️
* Implement a Kinesis-like streaming-service
* Single binary
* Easy to Host, Run & Operate (have you tried to run Kafka yourself?)
  * Kubernetes friendly

### Commands
> Available make commands

* `make build` - Builds the application with cargo
* `make build_release` - Builds the application with cargo, with release optimizations
* `make docker_test_watcher` - Runs funzzy on linux over docker-compose
* `make docs` - Generate the GitHub Markdown docs (At the moment only mermaid)
* `make format` - Formats the code according to cargo
* `make help` - Lists the available commands. Add a comment with '##' to describe a command.
* `make install` - Builds a release version and installs to your cago bin path
* `make run` - Runs the newly built
* `make test` - Tests all features
