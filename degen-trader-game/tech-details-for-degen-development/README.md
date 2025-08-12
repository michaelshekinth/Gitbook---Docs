# Tech Details for Degen Development

### Architecture Overview

Degen Trader is implemented as a fully decentralized web-based game on **Solana**, leveraging on-chain programs (smart contracts) for game logic, token handling, and staking, with Kingdom Coin (KC) as the in-game currency.&#x20;

The Degen Trader game logic is implemented in a Solana program using **Rust and Anchor**, enabling secure, transparent game sessions. The gameplay follows a **buy-low, sell-high crypto trading simulation** over a 30-day in-game session with randomized events, and integrates tightly with the KC tokenomics. Below is a step-by-step outline of the gameplay flow and how the smart contract manages each stage

**Session Initialization (Start Game)**

When a player decides to play, they initiate a new game session by calling the **`start_session`** instruction on the Degen Trader program.This requires a payment of **12.5 KC** entry fee from the player’s wallet. The program verifies the fee and immediately burns **10%** (i.e. **1.25 KC**) as per the deflationary policy. The remaining 90% (11.25 KC) is transferred into the game’s treasury or prize pool account (a PDA or designated token account) for future rewards, development, or prize distribution. On-chain enforcement of the fee and burn ensures every play contributes to token burn. The `start_session` handler may also create a **Game Session account** (PDA) to track this session’s state, including the player’s public key, session start time, and initial game state (starting virtual capital $2,000, current location = Bedroom, etc.).
