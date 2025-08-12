---
description: SOLar Mission
---

# Process of Building

### Technical Requirements

## Smart Contract Documentation (Rust, Anchor)

### Overview

This document provides an overview of the smart contract functionalities for managing player interactions in a hypothetical blockchain-based game. The smart contract is written in Rust using the Anchor framework. It includes account structures and various instructions to manage game sessions, player activities, and economy through Solana Blockchain.

### Accounts

#### PlayerState

* **Description**: This account is pivotal for storing all critical player-specific data.
* **Fields**:
  * `wallet`: The player's wallet address used for identifying their participation and activities.
  * `session data`: Information relevant to the player's ongoing or past game sessions.
  * `portfolio value`: The calculated value of the player's holdings within the game.
  * `points`: The current score or points that the player has accumulated.

#### GameConfig

* **Description**: Stores global configuration and economic parameters essential for game mechanics.
* **Fields**:
  * `entry fee`: The amount required for a player to enter a game session.
  * `burn %`: The portion of currency spent that is removed from circulation.
  * `prize %`: The percentage of entry fees or other pools reallocated as prizes.
  * `price RNG params`: Parameters for the random number generator impacting game variables.

### Instructions

#### initialize\_session

* **Purpose**: To set up the foundational state for a new game session.
* **Functionality**:
  * Burns a specified amount of KC.
  * Sets a starting balance for the player, preparing them for subsequent gameplay.

#### play\_turn

* **Purpose**: Represents a single action or move within the game session.
* **Functionality**:
  * Fetches a random number to determine price fluctuations and game events.
  * Updates the player's current holdings based on the results of their turn.

#### end\_session

* **Purpose**: To conclude the player's active game session.
* **Functionality**:
  * Calculates and records the final points achieved by the player.
  * Adjusts the player's staking tier based on performance, influencing future benefits or game conditions.

#### purchase\_item

* **Purpose**: Enables players to buy in-game items or resources.
* **Functionality**:
  * Deducts and burns 20% of the purchase cost in KC.
  * Allocates the remaining 80% of the purchase cost to the game treasury.



**On-chain KC token handling** (entry fees, burns, prize pool allocation)

**Provably fair RNG** for multiplier/crash point generation

\
&#x20;We use  **cryptographically secure randomness verifiable on-chain** — e.g., Switchboard VRF or Solana’s native randomness sources — so the crash outcomes can be proven fair after each round.

* **Session-based model**:
  * Each game session starts with **12.5 KC** (10% burned on-chain, rest distributed).
  * Player gets **2,000 in-game points** to place multiple bets in quick crash rounds.
  * Results, scores, and point totals need to be **recorded on-chain** at the session end for staking APR integration.
* **Frontend in React + Tailwind** with wallet connection (Phantom, Solflare, Sollet), handling game visuals and player interactions.
* **Smart contract** only handles:
  1. Taking the KC entry fee
  2. Burning 10% immediately
  3. Allocating the rest to the right pools (treasury, rewards, dev)
  4. Storing the player’s final score at session end for point conversion
  5. Optional: Skin/enhancement purchases with 20% burn.
* **Backend** in Node.js will:
  * Handle matchmaking and multiplayer leaderboards
  * Cache multiplier sequences for animation playback (though official outcomes are on-chain)
  * Relay VRF results from Solana to the frontend

### Backend Architecture (Node.js)

#### Leaderboard Cache

* **Description**: A caching mechanism to provide fast access to leaderboard data.

#### Solana Event Listening

* **Functionality**:
  * Actively listens for Solana blockchain events such as KC burns and point updates to stay in sync with player activities and economic changes.

#### REST API Endpoints

* **Purpose**: Facilitates communication and interaction between front-end and back-end components.

**GET /leaderboard**

* **Description**: Provides current leaderboard rankings.

**GET /prices**

* **Description**: Returns the latest in-game asset prices.

**POST /session**

* **Description**: Initiates a new game session for the player.

**POST /purchase**

* **Description**: Processes a transaction for purchasing in-game items or resources.

### Tokenomics Sanity Check

( According to the Token Contract )

* Entry Fee Burn: 1.25 KC/session → \~95M KC/year at 380K active players × 0.55 plays/day.
* Enhancements Burn: \~39M KC/year.
* Annual Burn: \~134M KC/year (deflationary).
* APR Pool: \~175M KC, \~8 years sustainability at avg 10% APR.

_**No mismatch detected — model is sustainable with deflation > inflation**_
