# Code Explanations

### **Overview**

This program implements the **Kingdom Coin (KC) core logic** for:

* Entry fees (with burn + treasury split)
* Staking & unstaking KC
* Claiming staking rewards
* Tracking gameplay points from different games (Solar & Degen)
* Tier-based staking boost
* Admin-controlled settings

It’s written in **Rust** using the **Anchor framework** for Solana and integrates with the **SPL Token Program** for KC token operations.

### **Key Constants**

```rust
const SECONDS_PER_YEAR: i64 = 31_536_000;
const BPS_DENOMINATOR: u64 = 10_000;
```

* **BPS (basis points)** system is used for percentages.
* `SECONDS_PER_YEAR` used to calculate APR-based rewards.

```rust
const MONTHLY_CAP_SOLAR: u64 = 50_000_000;
const MONTHLY_CAP_DEGEN: u64 = 50_000_000;
```

* Each game has a **monthly max points cap**.

```rust
const ENTRY_FEE_RAW: u64 = 12_500_000_000; // 12.5 KC
const ENTRY_BURN_BPS: u16 = 1000; // 10%
const UNSTAKE_BURN_BPS: u16 = 500; // 5%
```

* **Entry fee**: 12.5 KC (9 decimal places).
* 10% burned, 90% goes to treasury.
* Unstaking burns 5% of the amount.

### **Core Instructions**

#### **1. `initialize`**

* Admin-only.
* Sets **APR rates**, **tier thresholds**, **boosts**, and vault addresses.
* Records the starting month for point tracking.

#### **2. `pay_entry`**

* Called by a player to pay the **entry fee**.
* Splits payment:
  * Burn **10%**
  * Send **90%** to the game treasury
* Emits `EntryPaid` event.

#### **3. `stake`**

* Player stakes KC into the **staking vault**.
* If first stake, initializes a `UserStake` account for them.
* Calls `accrue_rewards_internal` to update pending rewards before changing stake.
* Adds staked amount to total.

#### **4. `claim`**

* Claims any pending rewards into player’s KC token account.
* Rewards come from the **rewards vault**.
* Uses PDA (`vault_authority`) to sign transfer.
* Emits `RewardsClaimed`.

#### **5. `unstake`**

* Removes staked KC.
* Burns 5% fee from vault.
* Sends 95% back to the user.
* Calls `accrue_rewards_internal` before updating.
* Emits `Unstaked`.

#### **6. `update_points`**

* Admin or authorized game/oracle updates monthly gameplay points for a user.
* Handles rollover to a new month (resets counters).
* Updates points for **Solar** or **Degen** games, respecting monthly caps.
* Calls `recompute_tier` to adjust staking tier and boosts.
* Emits `PointsUpdated`.

#### **7. `set_tiers` & `set_apr`**

* Admin-only functions to update tier thresholds, boosts, and APR values.

### **Helper Functions**

#### **`accrue_rewards_internal`**

* Calculates staking rewards since last update:
  * `stake_amount × (APR + tier boost) × time_elapsed / year`
* Updates `pending_rewards` and last accrual timestamp.
* Caps APR at `max_apr_bps`.

#### **`recompute_tier`**

* Determines tier based on total points (Solar + Degen).
* For higher tiers (3+), enforces **multi-game requirement** (≥20% points from each game).
* Updates `current_tier` and `current_boost_bps`.

#### **`mul_bps`**

* Multiplies an amount by basis points.

#### **`current_month_ts`**

* Converts a timestamp into a **month bucket** (30-day months for simplicity).

#### **`token_balance`**

* Reads balance of a token account.

***

### **Accounts**

The program uses **Anchor `#[account]` structs** to define and validate all accounts passed into instructions.

#### **`Config`**

* Stores global settings (admin, KC mint, vaults, APR rates, tier thresholds, boosts, epoch month).

#### **`UserStake`**

* Stores individual user staking data:
  * Staked amount, pending rewards
  * Gameplay points for each game
  * Current tier & boost
  * Last reward accrual timestamp

### **Events**

Events are emitted for off-chain indexing and tracking:

* **`EntryPaid`** → when a player pays entry fee.
* **`RewardsClaimed`** → when rewards are claimed.
* **`Unstaked`** → when a player unstakes KC.
* **`PointsUpdated`** → when game points are updated.

### **Error Codes**

Custom errors for clarity:

* `MathOverflow` → arithmetic overflow detected.
* `InvalidAmount` → zero/negative amount where not allowed.
* `InsufficientStake` → unstake amount > staked amount.

### **Program Flow Example**

**Example: Player Joins & Plays**

1. Player calls `pay_entry()` → burns 10% KC, sends 90% to treasury.
2. Player calls `stake()` → KC transferred to vault, stake recorded.
3. Game server calls `update_points()` monthly as player plays.
4. At intervals, player calls `claim()` to get KC rewards.
5. If they want out, they call `unstake()` → 5% burn, rest returned.

