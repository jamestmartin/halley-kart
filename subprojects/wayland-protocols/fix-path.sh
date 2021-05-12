#!/bin/bash
path=$(realpath --relative-to . "$1/include" | tr -d '\n')
mkdir -p "$path"
echo -n "$path"
