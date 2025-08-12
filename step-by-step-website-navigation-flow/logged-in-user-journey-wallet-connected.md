# Logged-In User Journey (Wallet Connected)

### Wallet Connection & Authentication

<figure><img src="../.gitbook/assets/_- visual selection (1).png" alt=""><figcaption></figcaption></figure>

he user has connected a Solana wallet (e.g. Phantom). The site recognizes the wallet and logs the user in, establishing an authenticated session tied to that wallet address. Secure wallet integration ensures all actions (swaps, game entries, staking) will require the user’s signature, preventing unauthorized transactions. With the wallet connected, additional UI elements become active – for instance, the user’s KC balance is displayed (initially 0 if they just joined) and their profile/dashboard sections are now accessible.

### Swapping SOL to KC

After connecting, the first step is acquiring **Kingdom Coin (KC)** to use on the platform. The website provides an integrated swap interface where the user can exchange Solana (SOL) for KC at the current rate (initially $0.04 per KC). The user enters the amount of SOL to swap and confirms the transaction in their wallet. Under the hood, this calls a Solana program or on-chain DEX to convert SOL to KC and credit the user’s wallet with KC tokens. The platform may recommend swapping a sufficient amount of KC (e.g. at least 500 KC) so the user can both play games and meet the minimum for staking rewards. Once the swap is confirmed on-chain, the user’s KC balance updates on the site. _Onboarding Tip:_ The site could prompt first-time swappers with guidance (e.g. “Swap SOL for KC to start playing – 12.5 KC per game session”) to ensure they have enough tokens for the next steps.

### **Playing Games (SOLar Mission & Degen Trader)**

With KC available, the user can now enter the games for real (non-demo) and start competing. Navigating to the **Games** section, they choose either _SOLar Mission_ or _Degen Trader_ from the game list. The game interface loads in-browser (a React-based embedded app) without leaving the site. To start a session, the user pays the entry fee of **12.5 KC** via a wallet transaction (the site will prompt for approval). Upon payment, the game session begins loading:
