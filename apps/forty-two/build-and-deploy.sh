set -eo pipefail
set -x # Debug output.

# ---------------------------------------------------------
# BUILD
# ---------------------------------------------------------

# Directory of this script.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Compile the intent set.
pintc "$SCRIPT_DIR/forty-two.pnt"

# Use `jq` to change the JSON from an object to a list.
# TODO: `pintc` should address this upstream: essential-contributions/pint#597.
INTENT_SET_JSON_FILE="$SCRIPT_DIR/forty-two.json"
jq '[.[]]' $INTENT_SET_JSON_FILE > tmp.json && mv tmp.json $INTENT_SET_JSON_FILE

# ---------------------------------------------------------
# SIGN
# ---------------------------------------------------------

# Create a keypair to sign with.
KEYPAIR_JSON=$(essential generate-keys)
PRIVATE_KEY_JSON=$(echo $KEYPAIR_JSON | jq -c ."private")

# Sign the single inner intent and update JSON.
SIGNED_INTENT_SET_JSON_FILE="$SCRIPT_DIR/forty-two-signed.json"
essential sign-intent-set --private-key-json "$PRIVATE_KEY_JSON" $INTENT_SET_JSON_FILE > $SIGNED_INTENT_SET_JSON_FILE

# ---------------------------------------------------------
# DEPLOY
# ---------------------------------------------------------

# Deploy the intent set. Assumes the following server port.
SERVER_PORT="45539"
JSON_DATA=$(jq . $SIGNED_INTENT_SET_JSON_FILE)
RESPONSE=$(curl -X POST -H "Content-Type: application/json" \
  -d "$JSON_DATA" \
  "http://localhost:$SERVER_PORT/deploy-intent-set")

# Retrieve the intent addresses (contains only the one intent address in this case).
INTENT_ADDRESSES=$(essential intent-addresses $INTENT_SET_JSON_FILE)
INTENT_ADDRESS=$(echo $INTENT_ADDRESSES | jq -c '.[0]')

# Before continuing, ensure that the response we got from the server when we
# deployed the intent set matches the INTENT_SET_CA we expect.
INTENT_SET_CA=$(echo $INTENT_ADDRESS | jq -c '."set"')

if [ "$RESPONSE" != "$INTENT_SET_CA" ]; then
  echo "Error: RESPONSE does not match INTENT_SET_CA"
  echo "RESPONSE: $RESPONSE"
  echo "INTENT_SET_CA: $INTENT_SET_CA"
  exit 1
fi

# ---------------------------------------------------------
# SOLVE
# ---------------------------------------------------------

# Construct a solution with the `42` decision var and the address we want to change the state to.
# TODO: This is super unwieldy - would be great if pintc could generate this.
# TODO: Don't use `Signed<Solution>`, instead just use `Solution`.
ANSWER="42"
SOLUTION=$(jq -n \
  --argjson intent_addr "$INTENT_ADDRESS" \
  --argjson answer "$ANSWER" \
'
{
  data: {
    data: [
      {
        intent_to_solve: $intent_addr,
        decision_variables: [
          {
            Inline: $answer
          }
        ]
      }
    ],
    state_mutations: [
      {
        pathway: 0,
        mutations: [
          {
            key: [0,0,0,0],
            value: 1
          }
        ]
      }
    ]
  },
  signature: [
    [
      227,149,64,152,61,122,243,188,139,161,53,210,43,86,106,204,89,249,201,75,200,88,214,81,248,111,37,27,148,225,87,74,110,213,26,68,171,171,18,221,207,212,83,56,94,250,152,9,44,100,237,37,49,208,239,95,229,91,202,99,66,13,148,225
    ],
    0
  ]
}')

# Submit the solution.
RESPONSE=$(curl -X POST -H "Content-Type: application/json" \
  -d "$SOLUTION" \
  "http://localhost:$SERVER_PORT/submit-solution")

# TODO: Convert the solution hash into expected hash format (base64?).

# Check the outcome of the solution.
# curl -X GET -H "Content-Type: application/json" \
#   "http://localhost:$SERVER_PORT/solution-outcome/NsFZ12tS4D5JY2NgfFlAIn9i9OBI3zRLBQFZvJe7o9c="
