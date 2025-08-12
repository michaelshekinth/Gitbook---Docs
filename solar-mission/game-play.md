---
description: SOLar Mission
---

# Game-Play

### **Entry Fee & Token Burn**

Each game session costs **12.5 KC** (Kingdom Coin) to play. The contract enforces that 10% of this fee (**1.25 KC**) is immediately burned (removed from supply) and the remaining 90% (**11.25 KC**) is transferred into the game’s treasury/prize pool accounts.

### **Session Tracking & Points**

&#x20;When a player starts a session (`start_session` instruction), a **Session PDA** (program-derived account) is created (or reinitialized) for that player to track the game state. The entry fee of 12.5 KC grants the player **2,000 points** to use in-game (per design, 12.5 KC = 2000 points). The session account records the player’s remaining points, active bets, and status (active or ended). Only one session can be active per player at a time – the contract will prevent starting a new session if the previous session hasn’t been properly ended (avoiding reentrancy or overlap issues).

### Bet Placement and Crash Rounds

The game allows the player to place one or two bets per round, with each bet between **10 and 1000 points**. The `place_bets` instruction locks in the bet amounts (deducting from the session’s point balance) and then triggers a new round. To determine the round’s outcome, the contract uses **verifiable randomness** – in practice via a Switchboard VRF oracle – to produce a random crash multiplier for that round, in a range from 1.00x up to a hard-capped maximum (10,000x). (The code is structured to request randomness and will receive a callback to finalize the round outcome in `resolve_round`.) This design ensures the outcome is **provably fair** and unpredictable. A short delay (e.g. 8–30 seconds as per game design) would be handled off-chain for the rocket animation, while on-chain the outcome is determined by the VRF response.

### Crash Outcome & Cash-Out Logic

For each bet, the player can optionally specify an **auto cash-out multiplier** (e.g. cash out at 2.0x) at the time of betting, or they can attempt to **manual cash-out** by calling a `cash_out` instruction during the round. The contract records these targets but does _not_ resolve the outcome until the random crash point is known. When `resolve_round` is called (after the VRF provides the crash point), the contract compares the crash multiplier to each bet’s cash-out target
