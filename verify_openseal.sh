#!/bin/bash
set -e

echo "🧪 Starting OpenSeal Integration Test..."

# 1. Unit Tests
echo "----------------------------------------"
echo "🛠️  Running Unit Tests..."
cargo test --workspace --quiet
echo "✅ Unit Tests Passed"

# 2. Build Binaries
echo "----------------------------------------"
echo "🏗️  Building Binaries..."
cargo build --quiet
CLI_BIN="./target/debug/openseal-cli"
RUNTIME_BIN="./target/debug/openseal-runtime"
echo "✅ Binaries Built"

# 3. Setup Mock Environment
echo "----------------------------------------"
TEST_DIR="test_env_$(date +%s)"
fs_source="$TEST_DIR/src"
fs_dist="$TEST_DIR/dist"
mkdir -p "$fs_source"

echo '{"message": "Hello OpenSeal", "status": "running"}' > "$fs_source/api.json"
echo "console.log('App Logic');" > "$fs_source/app.js"
echo "SECRET_KEY=12345" > "$fs_source/.env" # Should be ignored if we had .gitignore, but for now just files

# Create .gitignore to test ignore logic
echo ".env" > "$fs_source/.gitignore"

# 5. Run OpenSeal Run (Child Process Mode)
# We test the new 'openseal run' command which should spawn the app itself.
echo "----------------------------------------"
echo "🛡️  Starting OpenSeal Run (Child Mode)..."

# Create a mock Node.js app that listens on PORT
cat <<EOF > "$fs_source/server.js"
const http = require('http');
const port = process.env.PORT || 9090;
const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ message: "Hello OpenSeal Child", port: port }));
});
server.listen(port, () => {
    console.log(\`Mock App listening on port \${port}\`);
});
EOF

# Build first to get openseal.json with exec command
echo "📦 Building bundle with entry command..."
$CLI_BIN build --source "$fs_source" --output "$fs_dist" --exec "node server.js" > /dev/null

# Run the bundle
# Port 8081
$CLI_BIN run --app "$fs_dist" --port 8081 > "$TEST_DIR/runtime_child.log" 2>&1 &
RUNNER_PID=$!
sleep 5 # Wait for node to start

# 6. Test Request
echo "----------------------------------------"
echo "📨 Sending Request to OpenSeal Child Runner..."
RESPONSE=$(curl -s "http://127.0.0.1:8081/api.json")

echo "📥 Received Response (Raw):"
echo "$RESPONSE"

# 7. Verification using Python
echo "----------------------------------------"
VERIFY_SCRIPT="
import sys, json
try:
    data = json.load(sys.stdin)
    has_seal = 'openseal' in data and data['openseal'] is not None
    msg = data.get('result', {}).get('message')
    has_result = msg == 'Hello OpenSeal Child'
    internal_port = data.get('result', {}).get('port')
    
    if has_seal and has_result:
        print(f'✅ Child Process Verification PASSED.')
        print(f'   Seal Present: Yes')
        print(f'   Internal App Message: {msg}')
        print(f'   Internal Port: {internal_port} (Should be random/different from 8081)')
        sys.exit(0)
    else:
        print('❌ Verification FAILED')
        sys.exit(1)
except Exception as e:
    print(f'❌ JSON Parse Error: {e}')
    sys.exit(1)
"

echo "$RESPONSE" | python3 -c "$VERIFY_SCRIPT"
if [ $? -ne 0 ]; then
    echo "Logs:"
    cat "$TEST_DIR/runtime_child.log"
    kill $RUNNER_PID
    exit 1
fi

echo "----------------------------------------"
kill $RUNNER_PID
rm -rf "$TEST_DIR"
echo "🎉 All Integration Tests Passed!"
