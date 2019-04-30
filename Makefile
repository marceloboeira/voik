CARGO_BIN ?= `which cargo`
TARGET_PATH ?= `pwd`/target/release
BIN_VERSION ?= 0.1.0
BIN_NAME ?= loglady
BIN_PATH ?= $(TARGET_PATH)/$(BIN_NAME)

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
	@$(BIN_PATH)

.PHONY: install
install: build_release
	$(CARGO_BIN) install --force

.PHONY: test
test: format
	@$(CARGO_BIN) test
