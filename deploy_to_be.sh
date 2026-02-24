#!/bin/sh
npm run build
rm -rf ../rust-be-template/fe/*
find ./dist -type f ! -iname '*.png' ! -iname '*.jpg' \
  -exec sh -c 'zstd --ultra -22 -c "$1" > "$1.zst"' _ {} \;

cd ./dist
# Copy zstd-compressed files
find . -type f -name '*.zst' | while read file; do
  mkdir -p "../../rust-be-template/fe/$(dirname "$file")"
  cp "$file" "../../rust-be-template/fe/$file"
done
# Copy original .png and .jpg files as-is
find . -type f \( -iname '*.png' -o -iname '*.jpg' \) | while read file; do
  mkdir -p "../../rust-be-template/fe/$(dirname "$file")"
  cp "$file" "../../rust-be-template/fe/$file"
done
cd ..
