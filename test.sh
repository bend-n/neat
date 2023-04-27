#!/bin/bash
(cd core && cargo build "$@")
(cd tests && godot4 --headless)
