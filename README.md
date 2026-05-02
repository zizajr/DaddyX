# DaddyX by TicketDaddy

**Fan-Powered Event Pre-Financing on Solana**

> "Back the event. Earn from the night."

Built for the **[Colosseum Frontier Hackathon](https://colosseum.com/frontier)** — Consumer Applications track.

[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana)](https://solana.com)
[![Anchor](https://img.shields.io/badge/Anchor-0.30.x-blue)](https://www.anchor-lang.com)
[![Next.js](https://img.shields.io/badge/Next.js-14-black)](https://nextjs.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Whitepaper

The FeedbackFunding mechanism powering DaddyX is documented in full at:

**[daddyx.ticketdaddy.io/whitepaper](https://daddyx.ticketdaddy.io/whitepaper)**

Covers the mathematical model, tokenomics, smart contract architecture, settlement design, and the academic source (Dietz & Ngabonziza, 2019).

---

## What Is DaddyX?

DaddyX is a fan-powered event pre-financing protocol. Before an event happens, fans back it by purchasing DaddyX tokens. The mechanism works as follows:

- Each token purchase **step-prices** the next one by a fixed multiplier (Step Factor **[S]**)
- The outbid holder receives a **guaranteed fixed ROI** automatically on-chain (Payout Factor **[P]**)
- Final token holders at event close receive a **share of verified ticket revenue** reported by the TicketDaddy oracle

**For organizers:** Working capital before a single ticket is sold, released in milestone tranches.
**For fans:** A financial instrument for events they actually care about — guaranteed return if outbid, revenue share if they hold.

The mechanism is adapted from **FeedbackFunding** — a model co-authored as an academic paper by Florian Dietz and Gavin Ngabonziza (2019).

### The Math

```
New token price     =  C × S
Outbid holder gets  =  C × P    (guaranteed ROI = P − 1)
Organizer receives  =  C × (S − P) per purchase
Exploit prevention  =  cost to raise price = (B − A) × (S − P) / (S − 1)
```

Where `C` = current price, `S` = step factor, `P` = payout factor, `B` = new price, `A` = current price.

---

## What This Repo Contains

This repo was built entirely for the Colosseum Frontier Hackathon. It contains two related components of the **TicketDaddy Solana stack**:

### 1. TicketDaddy Solana Infrastructure
The foundational ticketing layer — built by **Kayondo Edward (Co-CTO)**:
- **NFT Ticket Minting** — each ticket is a 1:1 on-chain artifact on Solana, providing blockchain-verifiable proof of access
- **On-chain Ticket Sales** — ticket purchases processed on-chain with Phantom/Solflare wallet support
- **Event Escrow & Payouts** — organizer funds held in a Solana PDA escrow; 48-hour automated payout after event settlement (vs. the 7–30 day industry standard in East Africa)

### 2. DaddyX Protocol
The novel financial layer — the unique hackathon submission:
- **CreatorRegistry** — vetting and access control for approved event organizers
- **EventConfig + TokenState** — FeedbackFunding implementation in Rust/Anchor
- **EventEscrow** — campaign capital held and released via milestone tranches
- **Settlement Oracle** — post-event revenue reporting and distribution
- **Frontend** — Next.js 14 app with wallet integration, event discovery, organizer dashboard, campaign creation, whitepaper, and pitch deck

---

## Tech Stack

| Layer | Technology |
|---|---|
| Smart Program | Rust 1.75+, Anchor 0.30.x |
| Frontend | Next.js 14 (App Router), TypeScript |
| Styling | Tailwind CSS, shadcn/ui |
| Wallet | @solana/wallet-adapter (Phantom, Solflare) |
| Web3 | @solana/web3.js v1.x |
| Charts | Recharts |
| Database | Prisma + SQLite (off-chain event metadata) |
| Storage | IPFS (campaign details) |
| Math rendering | KaTeX (whitepaper) |
| Network | Solana devnet (mainnet-ready architecture) |

---

## Repo Structure

```
daddyx-solana/
├── programs/
│   └── daddyx/
│       └── src/
│           └── lib.rs          ← Anchor program (all instructions)
├── tests/
│   └── daddyx.ts               ← Full Anchor test suite
├── app/
│   ├── app/
│   │   ├── page.tsx            ← Landing page
│   │   ├── events/             ← Event discovery + detail pages
│   │   ├── dashboard/          ← Fan portfolio
│   │   ├── organizer/          ← Organizer dashboard
│   │   ├── creator/            ← Creator Program + campaign creation
│   │   ├── whitepaper/         ← Full whitepaper with KaTeX math
│   │   ├── pitch/              ← Pitch deck slides
│   │   └── admin/              ← Oracle + admin panel
│   ├── components/
│   ├── lib/
│   │   └── anchor/             ← Program IDL and client helpers
│   └── providers.tsx           ← Wallet adapter providers
├── scripts/
│   ├── deploy.ts               ← Devnet deployment
│   └── seed.ts                 ← Demo event seeding
├── prisma/
│   └── schema.prisma
├── Anchor.toml
├── Cargo.toml
└── package.json
```
## Smart Contract Architecture

### Accounts

| Account | PDA Seeds | Description |
| :---- | :---- | :---- |
| EventConfig | \[b"event", event\_id\] | Campaign parameters, oracle, step/payout factors, revenue |
| TokenState | \[b"token", event\_config, token\_id\] | Per-token owner, current price, purchase history |
| EventEscrow | \[b"escrow", event\_config\] | Holds organizer capital from token sales |
| CreatorProfile | \[b"creator", creator\_wallet\] | Approved creator status and event count |

### Instructions

| Instruction | Caller | Description |
| :---- | :---- | :---- |
| apply\_as\_creator | Anyone | Submit Creator Program application |
| approve\_creator | Platform admin | Whitelist an organizer to create campaigns |
| initialize\_event | Approved creator | Create EventConfig \+ TokenState accounts \+ EventEscrow |
| purchase\_token | Anyone | Buy a token — step-prices next, pays previous holder ROI |
| raise\_price | Current token owner | Raise current price (exploit-prevention formula applied) |
| request\_milestone\_release | Organizer | Request milestone fund release |
| approve\_milestone\_release | Platform admin | Transfer milestone tranche to organizer |
| report\_revenue | Oracle wallet | Post-event: record verified ticket revenue on-chain |
| claim\_revenue | Final token holder | Claim share of settled revenue |
| cancel\_event | Organizer / admin | Cancel event |
| claim\_refund | Token holder | Claim refund if event cancelled |

### Default Milestone Schedule

| Milestone | Release | Trigger |
| :---- | :---- | :---- |
| 1 | 25% of escrow | Campaign funding goal reached |
| 2 | 50% of escrow | 30 days before event date |
| 3 | Remaining | After report\_revenue confirmed |

---

## Getting Started

Install prerequisites: Rust, Solana CLI, Anchor 0.30.0, then run `npm install`.

### Local Development

    solana-test-validator
    anchor build
    anchor test
    cd app && npm run dev

### Deploy to Devnet

    solana config set --url devnet
    solana airdrop 2
    anchor deploy --provider.cluster devnet
    npx ts-node scripts/seed.ts

### Demo Events (Devnet)

| Event | Tokens | Initial Price | Step Factor | Revenue Share |
|---|---|---|---|---|
| Kigali Jazz Festival | 100 | 0.05 SOL | 1.5x | 20% |
| Uganda Netball League Finals | 100 | 0.02 SOL | 2.0x | 15% |
| Doha Electronic Night | 200 | 0.10 SOL | 1.5x | 25% |

### Connecting as a Fan (Demo)

1. Install Phantom (phantom.app) or Solflare (solflare.com)
2. Switch wallet to Solana Devnet
3. Request devnet SOL at faucet.solana.com
4. Visit daddyx.ticketdaddy.io

---

## Tests

Run with: `anchor test`

| Test | Description |
|---|---|
| Happy path | Full flow: initialize → 5 purchases → report revenue → claim |
| raise_price | Discount formula verification |
| Cancellation + refund | Cancel event, all holders refund correctly |
| Exploit prevention | Alternating wallets — verify no value extraction |
| Access control | Non-oracle cannot report revenue; non-owner cannot cancel |
| Double claim | Second claim fails with AlreadyClaimed |
| Premature claim | Claim before revenue reported fails with RevenueNotReported |

---

## Program IDs

| Network | Program ID |
|---|---|
| Devnet | [Update after deploy] |
| Mainnet-Beta | Not yet deployed |

## The Team

| Name | Role | Contribution |
| :---- | :---- | :---- |
| Gavin Ngabonziza | CEO & Product Lead | Product architecture, DaddyX protocol design, FeedbackFunding adaptation. GitHub: @zizajr |
| Bruce Bagarukayo | CTO | Backend infrastructure, DevOps, API integrations, Anchor program review. Built the first MTN Mobile Money SDK for East Africa. GitHub: @bbagarukayo |
| Kayondo Edward | Co-CTO | Solana program development, NFT ticket minting, escrow contracts, frontend (Next.js), wallet adapter integration. GitHub: @kayondoedward |

Academic credit: The FeedbackFunding mechanism is adapted from a model co-authored with Florian Dietz ([www.elody.com](http://www.elody.com/)). His original 2019 work is cited in the whitepaper.

---

---

## Getting Started

Install prerequisites: Rust, Solana CLI, Anchor 0.30.0, then run `npm install`.

### Local Development

    solana-test-validator
    anchor build
    anchor test
    cd app && npm run dev

### Deploy to Devnet

    solana config set --url devnet
    solana airdrop 2
    anchor deploy --provider.cluster devnet
    npx ts-node scripts/seed.ts

### Demo Events (Devnet)

| Event | Tokens | Initial Price | Step Factor | Revenue Share |
|---|---|---|---|---|
| Kigali Jazz Festival | 100 | 0.05 SOL | 1.5x | 20% |
| Uganda Netball League Finals | 100 | 0.02 SOL | 2.0x | 15% |
| Doha Electronic Night | 200 | 0.10 SOL | 1.5x | 25% |

### Connecting as a Fan (Demo)

1. Install Phantom (phantom.app) or Solflare (solflare.com)
2. Switch wallet to Solana Devnet
3. Request devnet SOL at faucet.solana.com
4. Visit daddyx.ticketdaddy.io

---

## Tests

Run with: `anchor test`

| Test | Description |
|---|---|
| Happy path | Full flow: initialize → 5 purchases → report revenue → claim |
| raise_price | Discount formula verification |
| Cancellation + refund | Cancel event, all holders refund correctly |
| Exploit prevention | Alternating wallets — verify no value extraction |
| Access control | Non-oracle cannot report revenue; non-owner cannot cancel |
| Double claim | Second claim fails with AlreadyClaimed |
| Premature claim | Claim before revenue reported fails with RevenueNotReported |

---

## Program IDs

| Network | Program ID |
|---|---|
| Devnet | [Update after deploy] |
| Mainnet-Beta | Not yet deployed |

## About TicketDaddy

TicketDaddy is East Africa's unified platform for events, travel, stays, and experiences.

* 600,000+ tickets and accreditations processed  
* 80+ business partners across Uganda, Rwanda, and Qatar  
* $400K in partner revenue  
* 14 months in operation  
* AyaHQ x Lisk '25 Accelerator | Web Summit Qatar Beta Startups

Website: ticketdaddy.io

DaddyX: daddyx.ticketdaddy.io

X: @ticketdaddy\_

---

## License

MIT

---

Built for the Colosseum Frontier Hackathon, May 2026\.

---

