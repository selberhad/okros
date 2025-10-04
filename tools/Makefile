# Makefile for development tools

.PHONY: all clean test

all: faketty.dylib

faketty.dylib: faketty.c
	gcc -shared -fPIC -o faketty.dylib faketty.c

test_isatty: test_isatty.c
	gcc -o test_isatty test_isatty.c

test: faketty.dylib test_isatty
	@echo "Testing faketty.dylib..."
	DYLD_INSERT_LIBRARIES=./faketty.dylib ./test_isatty
	@echo ""
	@echo "Running curses tests with fake TTY..."
	cd .. && DYLD_INSERT_LIBRARIES=./tools/faketty.dylib TERM=xterm-256color \
		cargo test --lib curses:: -- --test-threads=1

clean:
	rm -f faketty.dylib faketty.so test_isatty

.DEFAULT_GOAL := all
