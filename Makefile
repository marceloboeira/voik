## Cargo
CARGO_BIN ?= `which cargo`
TARGET_PATH ?= `pwd`/target/release
BIN_VERSION ?= 0.1.0
BIN_NAME ?= voik
BIN_PATH ?= $(TARGET_PATH)/$(BIN_NAME)

## Docker/Compose
COMPOSE ?= `which docker-compose`
COMPOSE_FILE ?= `pwd`/docker/compose.yml
COMMIT_LOG_PATH ?= `pwd`/commit_log/

## Testing
FUNZZY_BIN ?= `which funzzy`

## Docs
NPM ?= `which npm`
MERMAID ?= `which mmdc`
DOCS_PATH ?= `pwd`/docs

.PHONY: help
help: ## Lists the available commands
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: format ## Builds the application with cargo
	@$(CARGO_BIN) build

.PHONY: build_release
build_release: format ## Builds the application with cargo, with release optimizations
	@$(CARGO_BIN) build --release

.PHONY: format
format: ## Formats the code according to cargo
	@$(CARGO_BIN) fmt

.PHONY: run
run: build_release ## Runs the newly built
	@$(BIN_PATH)

.PHONY: install
install: build_release ## Builds a release version and installs to your cago bin path
	$(CARGO_BIN) install --force

.PHONY: test
test: ## Tests all features
	@$(CARGO_BIN) test --all-features
	@cd $(COMMIT_LOG_PATH) && $(CARGO_BIN) test --tests

.PHONY: test_watcher ## Starts funzzy, test watcher, to run the tests on every change
test_watcher:
	@$(FUNZZY_BIN)

.PHONY: docker_test_watcher
docker_test_watcher: ## Runs funzzy on linux over docker-compose
	@$(COMPOSE) -f $(COMPOSE_FILE) up

.PHONY: docs
docs: ## Generates the GitHub Markdown docs (At the moment only mermaid)
	@$(MERMAID) -w 800 -i $(DOCS_PATH)/architecture/graph.mmd -o $(DOCS_PATH)/architecture/graph.png
