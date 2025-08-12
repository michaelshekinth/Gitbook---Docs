# Basic Architecture & Tech Stack

## System Overview

### Frontend

* **Technologies**: React and Tailwind CSS
* **Components**:
  * Website
  * Game UIs
  * Staking Dashboard

### On-Chain (Solana)

* **Components**:
  * KC Token Management
  * Token Burns
  * Staking Mechanisms
  * Points/Caps Accumulation
  * Rewards Distribution
  * Governance Procedures

### Backend

* **Technologies**: Node.js and Rust
* **Features**:
  * Managing Game Sessions
  * Generating RNG Proofs
  * Maintaining Leaderboards
  * Implementing Referral Logic
  * Handling Webhooks

### Storage

*   **Mechanism**:

    * On-chain storage for key outcomes
    * Off-chain caching for performance optimization, particularly for leaderboards and telemetry data.



    <figure><img src="../.gitbook/assets/The ecosystem is designed as a hybrid on-chain_off-chain architecture_ - visual selection.png" alt=""><figcaption></figcaption></figure>

## First-Party Games (Developed In-House)

### Degen Trader

* **Objective**: Participants engage in a 30-day crypto trading simulation.
* **Inspiration**: Influenced by the mechanics of Drug Wars.

### SOLar Mission

* **Theme**: Space Exploration
* **Style**: Crash/multiplier rocket game
* **Feature**: Offers a provably fair game experience.

## Data Models (High-Level)

* **User**: Represents user profiles.
* **Wallet**: Details about users' wallets.
* **Stake**: Information on user stakes.
* **Points**: Data about points accumulated from different sources.
* **GameSession**: Records individual game sessions.
* **Round/Event**: Details specific rounds or events within a game.
* **LeaderboardEntry**: Entries on user leaderboard performance.
* **Proposal/Vote**: Data around proposals and governance votes.
* **ReferralEvent**: Records events related to referrals.

## Eventing

* **SwapCompleted**: Event triggered upon the completion of a token swap.
* **StakeUpdated**: Activated when there is an update in the staking status.
* **GameSessionEnded**: Marks the conclusion of a game session.
* **ReferralConverted**: An event indicating a successful referral conversion.
* **GovernanceFinalized**: Indicates the finalization of a governance process.
