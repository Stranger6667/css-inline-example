#!/bin/bash
set -ex

# Create a symlink for tokenizers-lib
ln -sf ../../css_inline css_inline
# Modify Cargo.toml to include this symlink
cp Cargo.toml Cargo.toml.orig
sed -i 's/\.\.\/\.\.\/css_inline/\.\/css_inline/' Cargo.toml
# Build the source distribution
python setup.py sdist
rm css_inline
mv Cargo.toml.orig Cargo.toml