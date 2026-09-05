#!/bin/bash
# test-client-credentials.sh
# Demonstrates the Client Credentials flow for machine-to-machine authentication
#
# Prerequisites:
#   - Keycloak running on localhost:8081
#   - Aperio running on localhost:3000
#   - Realm "aperio" with client "aperio-m2m" configured

set -euo pipefail

KEYCLOAK_URL="http://localhost:8081/realms/aperio/protocol/openid-connect/token"
APERIO_URL="http://localhost:3000/mcp"

CLIENT_ID="aperio-m2m"
CLIENT_SECRET="m2m-secret"

echo "=== Client Credentials Flow Test ==="
echo ""

echo "1. Requesting access token from Keycloak..."
echo "   POST $KEYCLOAK_URL"
echo "   grant_type=client_credentials"
echo "   client_id=$CLIENT_ID"
echo ""

TOKEN_RESPONSE=$(curl -sf -X POST "$KEYCLOAK_URL" \
  -d "grant_type=client_credentials" \
  -d "client_id=$CLIENT_ID" \
  -d "client_secret=$CLIENT_SECRET")

ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])")

echo "   ✓ Token received"
echo "   Token length: ${#ACCESS_TOKEN} chars"
echo ""

echo "2. Decoding token payload..."
echo "$ACCESS_TOKEN" | cut -d. -f2 | python3 -c "
import sys, base64, json
payload = sys.stdin.read().strip()
payload += '=' * (4 - len(payload) % 4)
decoded = base64.urlsafe_b64decode(payload)
data = json.loads(decoded)
print(f'   sub: {data.get(\"sub\", \"N/A\")}')
print(f'   iss: {data.get(\"iss\", \"N/A\")}')
print(f'   exp: {data.get(\"exp\", \"N/A\")}')
print(f'   scope: {data.get(\"scope\", \"N/A\")}')
"
echo ""

echo "3. Calling Aperio MCP endpoint with token..."
echo "   POST $APERIO_URL"
echo "   Authorization: Bearer <token>"
echo ""

MCP_RESPONSE=$(curl -sf http://localhost:3000/mcp \
  -X POST \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"m2m-client","version":"0.1"}},"id":1}')

echo "   ✓ MCP server responded"
echo "   Response: $MCP_RESPONSE" | head -c 200
echo "..."
echo ""

echo "=== Client Credentials Flow Complete ==="
echo ""
echo "Usage in production:"
echo "  export APERIO_CLIENT_ID=aperio-m2m"
echo "  export APERIO_CLIENT_SECRET=m2m-secret"
echo "  export APERIO_KEYCLOAK_URL=https://keycloak.example.com/realms/aperio/protocol/openid-connect/token"
