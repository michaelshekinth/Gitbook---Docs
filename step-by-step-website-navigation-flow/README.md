# Step By Step Website Navigation Flow

## Guest User Journey (No Wallet Connected)

### Landing on Homepage:

A new visitor arrives at SOLkingdom.com and sees a vibrant homepage introducing the Kingdom Coin ecosystem (KC) and its features. The hero banner invites the user to _“Join the SOL Kingdom! Swap SOL for KC and Play!”_ with a prominent Connect Wallet call-to-action. Game previews for SOLar Mission (crash game) and Degen Trader (trading sim) are showcased alongside live stats (e.g. active players, total KC staked, recent high scores), giving the user an overview of the platform’s activity and appeal.

<figure><img src="../.gitbook/assets/_- visual selection (4).png" alt=""><figcaption></figcaption></figure>

### **Exploring Without Login:**

While not logged in, the user can freely browse informational sections of the site. They can read about the games, tokenomics, and view leaderboards in a **read-only** mode. The site is designed to centralize access to all features (games, staking, referrals) even for unauthenticated users. Key navigation links (e.g. _Games_, _Staking Dashboard_, _Leaderboards_, _Referral Program_) are visible for exploration. However, any attempt to perform token transactions (such as swapping or staking) or to record game progress will prompt the user to connect a wallet.

### Demo Mode

Without a wallet, if the user tries to play a game, the platform offers a Demo Mode option for both SOLar Mission and Degen Trader. In demo mode, the user can play a session for free (no KC required) to get a feel for the gameplay mechanics, though no tokens or points are earned in this mode. This serves as a risk-free onboarding experience. The site may show an onboarding prompt or pop-up explaining that to play and earn rewards or points, a crypto wallet connection is required. Responsible gaming prompts (e.g. budget limits, warnings) are also provided to new users, encouraging mindful play and offering resources for help. At this stage, the user is encouraged to proceed by connecting their Solana wallet to fully access all features.

### Wallet Connect Trigger

When the guest user decides to engage beyond demo play – for example, clicking **“Connect Wallet”** or attempting to access the staking dashboard – the website initiates a Solana wallet connection flow. A supported wallet (such as Phantom or Solflare) will pop up for authentication. The user approves the connection, allowing the site to retrieve their wallet public key and authenticate them. From here on, the user’s session is associated with their wallet address (which serves as their account identity). The site is now ready to enable token swaps, game entries, and data tracking for this logged-in user.

