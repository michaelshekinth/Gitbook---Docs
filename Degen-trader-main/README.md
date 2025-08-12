# Degen Trader Game - Anchor Smart Contract

A Solana-based trading simulation game where players trade meme coins while random events affect the market.

## Features

### 🎮 Core Gameplay
- **Virtual Trading**: Buy and sell meme coins (DOGE, SHIB, PEPE, BONK, WIF, FLOKI)
- **Starting Capital**: Each player begins with $1,000
- **Daily Trading**: Process trades and advance to the next day
- **Portfolio Management**: Track holdings and calculate net worth

### 🎲 Random Events
- **On-chain Randomness**: Uses recent blockhashes for fair random generation
- **Event Types**:
  - 🚀 **Elon Tweet**: 5x price spike for affected coin
  - 💸 **Rug Pull**: Price drops to $0
  - 🏛️ **Tax Raid**: 50% price drop
  - 🐋 **Whale Movement**: 2x price increase
  - 📈 **Exchange Listing**: 3x price increase
  - ⚖️ **Regulation News**: 30% price drop
  - ⭐ **Celebrity Endorsement**: 2.5x price increase
  - 📉 **Market Crash**: 70% price drop
  - 🐂 **Bull Run**: 1.5x price increase
  - 📊 **Normal Day**: No major events

### 🏠 Location Progression
Players unlock new trading locations based on net worth:

| Location | Net Worth Requirement | Trading Fee |
|----------|----------------------|-------------|
| 🛏️ Bedroom | $0 - $999 | 2.0% |
| 🚗 Garage | $1,000 - $9,999 | 1.5% |
| 🏢 Office | $10,000 - $99,999 | 1.0% |
| 🛥️ Yacht | $100,000 - $999,999 | 0.5% |
| 🏙️ Penthouse | $1,000,000+ | 0.25% |

## Smart Contract Architecture

### Programs
- **Game State**: Global game configuration and coin prices
- **Player Session**: Individual player data and portfolio

### Key Instructions
1. `initialize_game`: Set up the game with initial coin prices
2. `create_player`: Register a new player with starting capital
3. `next_day`: Process trades, generate random events, update prices
4. `get_player_stats`: Retrieve current player statistics

### Data Structures
- **GameState**: Global game configuration
- **PlayerSession**: Individual player data
- **RandomEvent**: Generated events affecting gameplay
- **TradeOrder**: Buy/sell orders from players

## Installation & Setup

```bash
# Install dependencies
npm install

# Build the program
anchor build

# Deploy to localnet
anchor deploy

# Run tests
anchor test
```

## Usage Example

```typescript
// Initialize game (authority only)
await program.methods
  .initializeGame()
  .accounts({
    gameState: gameStatePda,
    authority: authority.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// Create player
await program.methods
  .createPlayer()
  .accounts({
    playerSession: playerSessionPda,
    gameState: gameStatePda,
    player: player.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([player])
  .rpc();

// Execute trades
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
    recentBlockhashes: SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
  })
  .signers([player])
  .rpc();
```

## Security Features

- **PDA-based Accounts**: Secure account derivation
- **Signer Verification**: Only players can modify their own sessions
- **Balance Validation**: Prevents negative balances and invalid trades
- **On-chain Randomness**: Fair and verifiable random events

## Testing

The test suite covers:
- Game initialization
- Player creation
- Trade processing
- Random event generation
- Location progression
- Error handling

Run tests with:
```bash
anchor test
```

## Events

The contract emits `DayCompleted` events containing:
- Player public key
- Current day
- Random event details
- Updated net worth
- Current location

## Error Handling

Custom error types:
- `PlayerNotActive`: Player session is inactive
- `GameNotActive`: Game is not running
- `InsufficientFunds`: Not enough cash for trade
- `InsufficientHoldings`: Not enough coins to sell
- `InvalidCoin`: Coin not supported
- `InvalidTradeAmount`: Invalid trade quantity

## Future Enhancements

- **Leaderboards**: Track top players
- **Tournaments**: Time-limited competitions
- **NFT Integration**: Special trading bonuses
- **Social Features**: Player interactions
- **Advanced Events**: More complex market scenarios