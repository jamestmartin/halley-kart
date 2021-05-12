#!/bin/sh
source_dir="$1"
project_build_dir="$2"
target_build_dir="$3"
base_name="$4"

XML_PATH=$(cat "$source_dir/protocols/$base_name")
wayland-scanner private-code < "$XML_PATH" > "$target_build_dir/$base_name-protocol.c"
wayland-scanner client-header < "$XML_PATH" > "$target_build_dir/$base_name-client-protocol.h"
mkdir -p "$project_build_dir/include"
cp "$target_build_dir/$base_name-client-protocol.h" "$project_build_dir/include/"
