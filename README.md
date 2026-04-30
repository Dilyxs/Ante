# ANTE Challenge Protocol

ANTE is a Solana-based challenge and bounty protocol for cryptography and math problems.

It includes:

- an on-chain Anchor program (`anchor_code`) that manages bounties, submissions, voting, refunds, and winner payouts
- a Rust backend service (`ante_backend`) that listens to on-chain events, stores them in Postgres, and exposes HTTP/WebSocket APIs
- a React frontend (`ante_frontend`) for wallet-connected users and real-time UI updates

## What the project does

Users can:

1. Get test ANTE tokens (admin-controlled faucet)
2. Deposit tokens into protocol balance
3. Create a bounty/poster with:
   - type (`OpenEnded` or `DirectAnswer`)
   - topic (`NumberTheory`, `CryptoPuzzle`, `ReverseEng`, etc.)
   - reward and submission cost
   - deadline
4. Submit encrypted answers before deadline
5. Reveal/decrypt answers after deadline
6. Handle outcomes:
   - post winner and distribute payout
   - vote for winner
   - refund answerers if publisher does not reveal
   - return funds if no submissions

The protocol emits Anchor events for all major actions, and the backend indexes those events for querying and live feeds.

## Repository structure

- `anchor_code/`
  Anchor workspace containing the Solana program (`programs/challenge-protocol`), TypeScript tests, and PDA/event logic.

- `ante_backend/`
  Axum + Tokio backend that:
  - subscribes to Solana logs (`logsSubscribe`)
  - decodes Anchor events
  - writes event rows into Postgres
  - serves REST endpoints for event queries
  - broadcasts live updates over WebSocket

- `ante_frontend/`
  Vite + React app with:
  - landing pages
  - Solana wallet adapter integration (Phantom/Solflare)
  - WebSocket connection to backend
  - dashboard shell for real-time protocol data

## Tech stack

- On-chain: Rust, Anchor, `anchor-spl`, Solana Token-2022 interfaces
- Backend: Rust, Axum, Tokio, Solana client/pubsub, SQLx, Postgres
- Frontend: React, Vite, Tailwind, Solana wallet adapter
- Tests: Mocha/Chai TypeScript tests in `anchor_code/tests`

## Core on-chain concepts

- Program ID (devnet): `7sGGy4oHLkKsfwzbb8fw2z8ZXH7ZMFhUeD62Z1y379tS`
- Protocol token mint PDA seed: `Ante`
- Vault authority PDA seed: `authority`
- Internal per-user balance PDA seed: `user_balance_info`
- Token decimals: `6`
- Faucet request sizes allowed: `1`, `2`, `4`, `8`

## Backend API highlights

Base URL (local): `http://localhost:3004`

- Health: `GET /`
- WebSocket: `ANY /websocket`
- Poster/event reads:
  - `/get_poster`
  - `/get_poster_answered`
  - `/get_all_answered_post`
  - `/get_poster_publish_answered`
  - `/get_all_poster_publish_answered`
  - `/get_answerer_decrypted_answer_posted`
  - `/get_all_answerer_decrypted_answer_posted`
  - `/get_poster_winner_posted`
  - `/get_all_poster_winner_posted`
  - `/get_publisher_not_responded`
  - `/get_all_publisher_not_responded`
  - `/get_vote_for_winner_posted`
  - `/get_all_vote_for_winner_posted`

See `ante_frontend/backend_api.md` for payload shapes.

## Local development (high level)

### 1) Anchor program

From `anchor_code/`:

- Install dependencies (`yarn` or `npm install`)
- Build/test:
  - `anchor build`
  - `anchor test`

### 2) Backend

From `ante_backend/`:

- Configure `.env` with DB and Solana websocket/RPC settings
- Ensure Postgres schema/tables exist for indexed events
- Run: `cargo run`

### 3) Frontend

From `ante_frontend/`:

- Configure `.env` for:
  - `VITE_SOLANA_RPC_URL`
  - `VITE_WEBSOCKET_URL`
- Run:
  - `npm install`
  - `npm run dev`

## Current status

The repo already has broad instruction and test coverage for:

- initialization
- token mint/deposit/withdraw
- poster creation
- answer submission
- answer reveal
- winner posting
- voting
- refund flow

Some backend winner-selection logic appears in progress, while core indexing and query routes are in place.

## Goal

ANTE aims to provide a transparent, programmable, and auditable bounty system for high-value cryptographic problem solving on Solana.
