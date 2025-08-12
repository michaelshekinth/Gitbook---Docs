# Token Swap

## **Swap**

#### **Quotes**

* Users can swap SOL → KC directly on the SOLkingdom.com site via the wallet connect interface.
* The quote endpoint should:
  * Accept the input token (e.g., SOL), amount, and output token (KC).
  * Return:
    * Exchange rate (KC per SOL, factoring in current DEX pool price on Raydium or Jupiter aggregator).
    * Estimated KC to be received.
    * Estimated slippage.
    * Fee breakdown (if applicable).
* Source of rates: Solana DEX aggregator (e.g., Jupiter API).

## **Execute Swap**

* Once the user approves the transaction in their connected wallet:
  * Smart contract executes the SOL → KC swap via Raydium/Jupiter route.
  * On completion:
    * KC is sent to the user’s wallet.
    * If holding ≥500 KC, auto-stake is triggered per tokenomics (8% base APR).
    * Transaction is recorded for leaderboards/rewards if swap bonuses are running.
* 10% of gameplay entry fees are burned, but swaps themselves have no burn — they increase KC circulation and staking.

#### **Status**

* Query the blockchain (via Solana RPC) for:
  * Transaction hash status (pending, confirmed, finalized).
  * KC balance update in the user’s wallet.
* Site shows a visual confirmation ("Swap Complete") and staking status update.
