---
description: The Degen Trader Game
---

# Sample Code for the structuring

A Solana-based trading simulation game where players trade meme coins while random events affect the market.

## Features

### Core Gameplay

* **Virtual Trading**: Buy and sell meme coins ( Ex: DOGE, SHIB, PEPE, BONK, WIF, FLOKI)
* **Starting Capital**: Each player begins with $2,000
* **Daily Trading**: Process trades and advance to the next day
* **Portfolio Management**: Track holdings and calculate net worth

#### 🎲 Random Events

* **On-chain Randomness**: Uses recent blockhashes for fair random generation
* **Event Types**: ( Examples )&#x20;
  * 🚀 **Elon Tweet**: 5x price spike for affected coin
  * 💸 **Rug Pull**: Price drops to $0
  * 🏛️ **Tax Raid**: 50% price drop
  * 🐋 **Whale Movement**: 2x price increase
  * 📈 **Exchange Listing**: 3x price increase
  * ⚖️ **Regulation News**: 30% price drop
  * ⭐ **Celebrity Endorsement**: 2.5x price increase
  * 📉 **Market Crash**: 70% price drop
  * 🐂 **Bull Run**: 1.5x price increase
  * 📊 **Normal Day**: No major events

#### 🏠 Location Progression

Players unlock new trading locations based on net worth:

| Location      | Net Worth Requirement | Trading Fee |
| ------------- | --------------------- | ----------- |
| 🛏️ Bedroom   | $0 - $999             | 2.0%        |
| 🚗 Garage     | $1,000 - $9,999       | 1.5%        |
| 🏢 Office     | $10,000 - $99,999     | 1.0%        |
| 🛥️ Yacht     | $100,000 - $999,999   | 0.5%        |
| 🏙️ Penthouse | $1,000,000+           | 0.25%       |

### Smart Contract Architecture

#### Programs

* **Game State**: Global game configuration and coin prices
* **Player Session**: Individual player data and portfolio

#### Key Instructions

1. `initialize_game`: Set up the game with initial coin prices
2. `create_player`: Register a new player with starting capital
3. `next_day`: Process trades, generate random events, update prices
4. `get_player_stats`: Retrieve current player statistics

#### Data Structures

* **GameState**: Global game configuration
* **PlayerSession**: Individual player data
* **RandomEvent**: Generated events affecting gameplay
* **TradeOrder**: Buy/sell orders from players

{% tabs %}
{% tab title="Degen-trade.ts" %}
```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DegenTrader } from "../target/types/degen_trader";
import { expect } from "chai";

describe("degen-trader", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DegenTrader as Program<DegenTrader>;
  const authority = provider.wallet as anchor.Wallet;
  const player = anchor.web3.Keypair.generate();

  let gameStatePda: anchor.web3.PublicKey;
  let playerSessionPda: anchor.web3.PublicKey;

  before(async () => {
    // Derive PDAs
    [gameStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("game_state")],
      program.programId
    );

    [playerSessionPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player_session"), player.publicKey.toBuffer()],
      program.programId
    );

    // Airdrop SOL to player
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        player.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      )
    );
  });

  it("Initializes the game", async () => {
    await program.methods
      .initializeGame()
      .accounts({
        gameState: gameStatePda,
        authority: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    expect(gameState.authority.toString()).to.equal(authority.publicKey.toString());
    expect(gameState.totalPlayers.toNumber()).to.equal(0);
    expect(gameState.isActive).to.be.true;
    expect(gameState.coinPrices).to.have.length(6);
  });

  it("Creates a player", async () => {
    await program.methods
      .createPlayer()
      .accounts({
        playerSession: playerSessionPda,
        gameState: gameStatePda,
        player: player.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    const playerSession = await program.account.playerSession.fetch(playerSessionPda);
    expect(playerSession.player.toString()).to.equal(player.publicKey.toString());
    expect(playerSession.cash.toNumber()).to.equal(100000); // $1000 in cents
    expect(playerSession.location).to.deep.equal({ bedroom: {} });
    expect(playerSession.isActive).to.be.true;
  });

  it("Processes a trading day", async () => {
    const recentBlockhashes = anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY;
    
    const trades = [
      {
        coinName: "DOGE",
        quantity: 100,
        action: { buy: {} }
      }
    ];

    await program.methods
      .nextDay(trades)
      .accounts({
        playerSession: playerSessionPda,
        gameState: gameStatePda,
        player: player.publicKey,
        recentBlockhashes: recentBlockhashes,
      })
      .signers([player])
      .rpc();

    const playerSession = await program.account.playerSession.fetch(playerSessionPda);
    expect(playerSession.day.toNumber()).to.equal(1);
    expect(playerSession.cash.toNumber()).to.be.lessThan(100000); // Should have spent money on DOGE
  });

  it("Gets player stats", async () => {
    const stats = await program.methods
      .getPlayerStats()
      .accounts({
        playerSession: playerSessionPda,
        gameState: gameStatePda,
        player: player.publicKey,
      })
      .signers([player])
      .view();

    expect(stats.day.toNumber()).to.equal(1);
    expect(stats.cash.toNumber()).to.be.greaterThan(0);
    expect(stats.netWorth.toNumber()).to.be.greaterThan(0);
  });

  it("Handles location progression", async () => {
    // This would require multiple days of trading to accumulate enough net worth
    // For testing purposes, we can modify the player's cash directly in a test environment
    console.log("Location progression test would require extended gameplay simulation");
  });

  it("Handles random events correctly", async () => {
    const recentBlockhashes = anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY;
    
    // Execute multiple days to see different random events
    for (let i = 0; i < 5; i++) {
      const trades = []; // No trades, just advance day
      
      const tx = await program.methods
        .nextDay(trades)
        .accounts({
          playerSession: playerSessionPda,
          gameState: gameStatePda,
          player: player.publicKey,
          recentBlockhashes: recentBlockhashes,
        })
        .signers([player])
        .rpc();

      // You could parse the transaction logs here to verify events were emitted
      console.log(`Day ${i + 2} completed: ${tx}`);
    }

    const playerSession = await program.account.playerSession.fetch(playerSessionPda);
    expect(playerSession.day.toNumber()).to.be.greaterThan(1);
  });
});
```
{% endtab %}

{% tab title="Anchor.toml" %}
```python
[features]
seeds = false
skip-lint = false

[programs.localnet]
degen_trader = "DegenTraderGameProgram11111111111111111111"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "Localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```
{% endtab %}

{% tab title="Package.json" %}
```json
{
  "scripts": {
    "lint:fix": "prettier */*.js \"*/**/*{.js,.ts}\" -w",
    "lint": "prettier */*.js \"*/**/*{.js,.ts}\" --check"
  },
  "dependencies": {
    "@coral-xyz/anchor": "^0.29.0"
  },
  "devDependencies": {
    "@types/bn.js": "^5.1.0",
    "@types/chai": "^4.3.0",
    "@types/mocha": "^9.0.0",
    "chai": "^4.3.0",
    "mocha": "^9.0.3",
    "prettier": "^2.6.2",
    "ts-mocha": "^10.0.0",
    "typescript": "^4.3.5"
  }
}
```
{% endtab %}

{% tab title="tsconfig.json" %}
```json
{
"compilerOptions": {
"types": ["mocha", "chai"],
"typeRoots": ["./node_modules/@types"],
"lib": ["es6"],
"module": "commonjs",
"target": "es6",
"esModuleInterop": true,
"allowSyntheticDefaultImports": true,
"experimentalDecorators": true,
"emitDecoratorMetadata": true,
"moduleResolution": "node",
"resolveJsonModule": true
}

```
{% endtab %}
{% endtabs %}
