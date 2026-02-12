#!/bin/bash
# Test script for new features

echo "Testing Advanced Features..."
echo "============================"
echo ""

# Build the project
echo "Building project..."
cargo build --release

# Run the advanced features example
echo ""
echo "Running advanced features demo..."
./target/release/rsedsim run examples/advanced_features.yaml -o /tmp/advanced_output.csv

# Check output
if [ -f /tmp/advanced_output.csv ]; then
    echo ""
    echo "Success! Output generated at /tmp/advanced_output.csv"
    echo ""
    echo "Sample output (first 10 lines):"
    head -10 /tmp/advanced_output.csv
    echo ""
    echo "Statistics:"
    echo "  Total lines: $(wc -l < /tmp/advanced_output.csv)"
    echo "  Columns: $(head -1 /tmp/advanced_output.csv | tr ',' '\n' | wc -l)"
else
    echo "Error: Output file not generated"
    exit 1
fi

echo ""
echo "Testing completed successfully!"
