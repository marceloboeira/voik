CARGO_BIN ?= `which cargo`
BIN_VERSION ?= 0.1.0
FUNZZY_BIN ?= `which funzzy`
COMPOSE ?= `which docker-compose`
COMPOSE_FILE ?= `pwd`/docker/compose.yml

.PHONY: build
build: format
	@$(CARGO_BIN) build

.PHONY: build_release
build_release: format
	@$(CARGO_BIN) build --release

.PHONY: format
format:
	@$(CARGO_BIN) fmt

.PHONY: run
run: build
	@$(CARGO_BIN) run --release --bin voik

.PHONY: benchmark
benchmark:
	@$(CARGO_BIN) run --release --bin voik_benchmark

.PHONY: install
install: build_release
	$(CARGO_BIN) install --force

.PHONY: test
test:
	@$(CARGO_BIN) test --all --verbose

.PHONY: test_watcher
test_watcher:
	@$(FUNZZY_BIN)

.PHONY: docker_test_watcher
docker_test_watcher:
	@$(COMPOSE) -f $(COMPOSE_FILE) up
