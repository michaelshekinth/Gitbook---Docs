# Smart Contract Explanation

### **GameConfig (PDA)**

* `kc_mint`: the KC SPL mint
* `treasury_vault`: KC ATA holding the 90% fee (prizes/treasury)
* `authority`: admin wallet
* `bump`: PDA bump

### **Session (PDA per user)**

* `active`, `round_active`
* `points` (your spendable points)
* `bet1`, `bet2` (each `Bet { amount, cashout_multiplier: Option<u32>, cashed_out: bool }`)

### **UserProfile (PDA per user)**

* `points_this_month` (cumulative, capped at 50M)
* `best_session_points`
* `session_count`

## Instruction flows, step-by-step

#### 1) Initialize

* Seeds: `["config"]` → creates the **config** PDA.
* Also creates the **treasury\_vault** ATA for the config as owner (so program can accumulate fees).
* Stores KC mint, treasury vault, admin pubkey.

#### 2) start\_session

* Guards: session must not be already active.
* **Burn 1.25 KC** from the user’s KC ATA.
* **Transfer 11.25 KC** from the user’s KC ATA to the **treasury\_vault**.
* Initializes session state: `active=true`, `points=2000`, no active round, clears bets.
* Increments `profile.session_count`.

#### 3) place\_bets

* Guards: session must be active; **no round currently active**.
* Validates bet amounts `[10, 1000]`, sums them, **deducts from session points**.
* Stores bet(s) with optional **auto cash-out** targets (`u32` scaled by 100).
* Sets `round_active=true`.
* **Note:** You’d trigger a **VRF request** (e.g., Switchboard) here; the oracle later calls `resolve_round`.

#### 4) cash\_out

* Guards: session active, round active.
* User picks bet index (1 or 2) and a desired **target multiplier** (scaled).
* If an auto target already exists, this lets the user set an **earlier** (smaller) manual target.
* Marks the bet as `cashed_out = true`. (Actual payout is handled in `resolve_round`.)

#### 5) resolve\_round

* **IMPORTANT:** In production, this must be called by the **VRF oracle/callback**, not just anybody.
* Demo logic derives a pseudo-random `crash_multiplier` from `Clock::get().unix_timestamp` (predictable — replace with VRF!).
* For each bet:
  * If `Some(target)` and `target ≤ crash_multiplier`, payout = `amount * target / 100` and add to `session.points`.
  * Else the bet is lost (points already deducted).
* Clears bets, sets `round_active = false`. If no points left, sets `session.active = false`.

#### 6) end\_session

* Guards: session active, **no round in progress**.
* Adds `final_points` to `profile.points_this_month` with a **50M cap**.
* Updates `best_session_points` if needed.
* Resets/clears session and deactivates it.

## Helpers & types

* `process_betOutcome` (helper): computes payout vs. crash multiplier (rename to `process_bet_outcome` to match Rust style).
* `Bet`: includes `amount`, optional `cashout_multiplier`, and a `cashed_out` marker.
* `ErrorCode` enums provide clean user-facing messages.

##
