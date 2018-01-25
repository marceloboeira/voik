CARGO_BIN = `which cargo`
TARGET_PATH = `pwd`/target/debug
BIN_NAME = loglady

.PHONY: build
build:
	$(CARGO_BIN) build

.PHONY: run
run: build
	$(TARGET_PATH)/$(BIN_NAME)
