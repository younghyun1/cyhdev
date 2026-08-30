#!/bin/bash
cd solid-csr-spa-template

file="src/pages/about.tsx"

# Find all unique shields.io URLs
urls=$(grep -oP 'https://img\.shields\.io/badge/[^"]+' "$file" | sort | uniq)

for url in $urls; do
    # Extract the part between /badge/ and ?
    name_part=$(echo "$url" | sed -E 's/.*\/badge\/([^?]+).*/\1/')
    
    # Generate filename by replacing %20 with _
    filename=$(echo "$name_part" | sed 's/%20/_/g').svg
    
    echo "Downloading $url to public/badges/$filename"
    curl -sL -A "Mozilla/5.0" "$url" -o "public/badges/$filename"
    
    # Escape URL for sed replacement
    escaped_url=$(echo "$url" | sed 's/[\/&]/\\&/g')
    
    # Replace in file
    sed -i "s/\"$escaped_url\"/\"\/badges\/$filename\"/g" "$file"
done
