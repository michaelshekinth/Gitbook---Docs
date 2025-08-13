# Step By Step Website Navigation Flow

## Guest User Journey (No Wallet Connected)

### Landing on Homepage:

A new visitor arrives at SOLkingdom.com and sees a vibrant homepage introducing the Kingdom Coin ecosystem (KC) and its features. The hero banner invites the user to _“Join the SOL Kingdom! Swap SOL for KC and Play!”_ with a prominent Connect Wallet call-to-action. Game previews for SOLar Mission (crash game) and Degen Trader (trading sim) are showcased alongside live stats (e.g. active players, total KC staked, recent high scores), giving the user an overview of the platform’s activity and appeal.

<figure><img src="../.gitbook/assets/_- visual selection (4).png" alt=""><figcaption></figcaption></figure>

### **Exploring Without Login:**

While not logged in, the user can freely browse informational sections of the site. They can read about the games, tokenomics, and view leaderboards in a **read-only** mode. The site is designed to centralize access to all features (games, staking, referrals) even for unauthenticated users. Key navigation links (e.g. _Games_, _Staking Dashboard_, _Leaderboards_, _Referral Program_) are visible for exploration. However, any attempt to perform token transactions (such as swapping or staking) or to record game progress will prompt the user to connect a wallet.

### Demo Mode

For new users without a connected wallet, the platform offers a Demo Mode option for both SOLar Mission and Degen Trader. This mode provides a free, no-KC-required introduction to gameplay mechanics, allowing players to experience the platform without any financial commitment.

* When a user attempts to start a game without a connected wallet, the system displays an **Onboarding Prompt** explaining Demo Mode, alongside a clear note that **tokens, points, and APR boosts cannot be earned** in this mode.
* Players can instantly launch the game in demo form without needing to sign up or deposit.
* **Responsible Gaming prompts** are shown, highlighting budget tips, mindful play, and links to responsible gaming resources.

**SOLar Mission (Crash Game)**

* Players receive a 1-minute demo session with standard game visuals and mechanics.
* They start with a fixed demo points balance, can place bets, and see how multipliers, cash-outs, and crashes work.
* No KC is spent, no burns occur, and no leaderboard updates are made.

**Degen Trader (Trading Simulation)**

* Players can try a 15 in-game day simulation in Demo Mode (half of the standard 30-day paid session).
* They start with $2,000 virtual in-game currency, can trade coins, react to random events, and experience the portfolio growth mechanic.
* At the end of 15 days, a summary screen shows their “what-if” earnings had they been playing the live mode.

### Wallet Connect Trigger

When the guest user decides to engage beyond demo play – for example, clicking **“Connect Wallet”** or attempting to access the staking dashboard – the website initiates a Solana wallet connection flow. A supported wallet (such as Phantom or Solflare) will pop up for authentication. The user approves the connection, allowing the site to retrieve their wallet public key and authenticate them. From here on, the user’s session is associated with their wallet address (which serves as their account identity). The site is now ready to enable token swaps, game entries, and data tracking for this logged-in user.

