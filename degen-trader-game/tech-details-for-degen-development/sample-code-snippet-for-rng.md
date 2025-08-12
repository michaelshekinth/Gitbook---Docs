# Sample Code Snippet for RNG

Once the session is started and the player has their virtual $2,000 capital, the gameplay proceeds through 30 in-game days (turns) of trading and events. For a fully on-chain implementation, each day’s actions can be processed by calling a `next_day` instruction

### **Price Fluctuation:**&#x20;

The program uses a secure random number generator or oracle (e.g., Solana block hash or Chainlink VRF) to determine new prices for each of the virtual cryptocurrencies in the game. Prices can randomly fluctuate between 1x to 10x of the previous value, with rare extreme events up to 100x pumps or crashes.

### **Oracle + RNG mix**

### **Anchor Program**

```rust
// Some codeuse anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use anchor_lang::solana_program::sysvar::clock::Clock;

declare_id!("GAME111111111111111111111111111111111111111");

#[program]
pub mod degen_trader_rng {
    use super::*;

    pub fn init_game_state(ctx: Context<InitGameState>, coins: Vec<String>, base_prices: Vec<u64>) -> Result<()> {
        require!(coins.len() == base_prices.len(), DegenError::LengthMismatch);
        let state = &mut ctx.accounts.state;
        state.admin = ctx.accounts.admin.key();
        state.last_slot = Clock::get()?.slot;
        state.coins = coins;
        state.prices = base_prices; // e.g., store as fixed-point (e.g., 1e6)
        Ok(())
    }

    /// Called after VRF fulfillment to update prices.
    /// `random_seed` is provided by the oracle callback (32 bytes).
    pub fn apply_random_prices(ctx: Context<ApplyRandomPrices>, random_seed: [u8; 32]) -> Result<()> {
        let state = &mut ctx.accounts.state;

        // Map each coin to a fresh u128 derived from the main seed
        for (i, p) in state.prices.iter_mut().enumerate() {
            let coin_seed = derive_child_seed(&random_seed, i as u64);
            let (mult_bps, crash) = sample_multiplier_basis_points(coin_seed);
            // crash == true means apply a crash direction (inverse of pump)
            // mult_bps is in basis points relative to 1.00x (e.g., 10_000 = 1.0x, 25_000 = 2.5x)

            let new_price = if crash {
                // floor to at least 1 “tick” to avoid going to 0
                (*p as u128)
                    .saturating_mul(10_000u128)
                    .checked_div(mult_bps.max(1) as u128)
                    .unwrap_or(*p as u128)
            } else {
                (*p as u128)
                    .saturating_mul(mult_bps as u128)
                    .checked_div(10_000u128)
                    .unwrap_or(*p as u128)
            };

            *p = u64::try_from(new_price.min(u128::from(u64::MAX))).unwrap();
        }

        state.last_slot = Clock::get()?.slot;
        Ok(())
    }
}

/// Derive a per-coin child seed from the top-level VRF seed
fn derive_child_seed(parent: &[u8; 32], idx: u64) -> [u8; 32] {
    // keccak(parent || idx_be)
    let mut bytes = [0u8; 40];
    bytes[..32].copy_from_slice(parent);
    bytes[32..].copy_from_slice(&idx.to_be_bytes());
    keccak::hash(&bytes).0
}

/// Sampling rule:
/// - ~98.5% of outcomes: 1.00x–10.00x (uniform or near-uniform)
/// - ~1.0% chance: 0.10x–0.50x crash (severe dump)
/// - ~0.5% chance: 10.01x–100.00x pump (moon)
///
/// Returns (multiplier_in_basis_points, crash_direction)
fn sample_multiplier_basis_points(seed32: [u8; 32]) -> (u32, bool) {
    // Convert first 16 bytes to u128
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&seed32[0..16]);
    let r = u128::from_be_bytes(buf); // 0..2^128-1
    let denom = u128::MAX;

    // Map to 0..1e6 for easy % buckets
    let rppm = ((r.saturating_mul(1_000_000)) / denom) as u32; // 0..999_999

    match rppm {
        0..=984_999 => {
            // common: 1.00x–10.00x, uniform
            // map 0..984_999 into [1.00x, 10.00x]
            let span_bps = 100_000 - 10_000; // 10.00x = 100_000 bps, 1.00x = 10_000 bps
            let scaled = 10_000 + ((rppm as u128 * span_bps as u128) / 985_000u128) as u32;
            (scaled.max(10_000), false)
        }
        985_000..=994_999 => {
            // crash: 0.10x–0.50x
            let span_bps = 5_000 - 1_000; // 0.50x to 0.10x
            let inner = rppm - 985_000;   // 0..9_999
            let scaled = 1_000 + ((inner as u128 * span_bps as u128) / 10_000u128) as u32;
            (scaled.max(1_000), true) // mark as crash_dir
        }
        _ => {
            // moon: 10.01x–100.00x
            let span_bps = 1_000_000 - 100_100; // 100.00x to 10.01x
            let inner = rppm - 995_000;         // 0..4_999
            let scaled = 100_100 + ((inner as u128 * span_bps as u128) / 5_000u128) as u32;
            (scaled.max(100_100), false)
        }
    }
}

#[derive(Accounts)]
pub struct InitGameState<'info> {
    #[account(init, payer = admin, space = 8 + GameState::MAX_SIZE)]
    pub state: Account<'info, GameState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApplyRandomPrices<'info> {
    #[account(mut, has_one = admin)]
    pub state: Account<'info, GameState>,
    pub admin: Signer<'info>, // you can replace with a VRF authority PDA checked by seeds
}

#[account]
pub struct GameState {
    pub admin: Pubkey,
    pub last_slot: u64,
    pub coins: Vec<String>,
    pub prices: Vec<u64>, // fixed-point or integer ticks
}
impl GameState {
    pub const MAX_COINS: usize = 32;
    pub const MAX_SYMBOL_BYTES: usize = 12;
    pub const MAX_SIZE: usize = 32 + 8 + (4 + Self::MAX_COINS * (4 + Self::MAX_SYMBOL_BYTES)) + (4 + Self::MAX_COINS * 8);
}

#[error_code]
pub enum DegenError {
    #[msg("coins and base_prices must be same length")]
    LengthMismatch,
}

```
