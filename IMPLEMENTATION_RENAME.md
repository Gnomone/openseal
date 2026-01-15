# Implement Protocol Rename: `nonce` -> `wax`

**Goal**: Rename `nonce` to `wax` in code and docs to align with OpenSeal's metaphorical branding.

## User Review Required
> [!WARNING]
> Breaking Protocol Change.
> JSON Input/Output field `nonce` will become `wax`.
> Header `X-OpenSeal-Nonce` will become `X-OpenSeal-Wax`.

## Proposed Changes

### `openseal-core`
#### [MODIFY] [lib.rs](file:///root/highpass/openseal/crates/openseal-core/src/lib.rs)
- Rename `Seal.nonce` struct field to `Seal.wax`.
- Update `compute_a_hash` and `compute_b_hash` signatures to take `wax`.

### `openseal-runtime`
#### [MODIFY] [lib.rs](file:///root/highpass/openseal/crates/openseal-runtime/src/lib.rs)
- Update JSON Output key from `nonce` to `wax`.
- Update Header key from `X-OpenSeal-Nonce` to `X-OpenSeal-Wax`.
- Update variable names `nonce_hex` -> `wax_hex`.

## Verification Plan

### Manual Verification
1.  Run `openseal run --app ...`
2.  Send a request with JSON body `{ "openseal": { "wax": "challenge_123" }}`.
3.  Verify Response contains `openseal.wax` matching input.
4.  Verify Response Header contains `x-openseal-wax`.
