# Staking

*   **Entry Fees and Burns (Deflationary Mechanics):** Each game session (both Degen Trader and SOLar Mission) costs **12.5 KC** with **1.25 KC burned**. With a large user base, this creates significant burn pressure:

    * Assuming **380,000 active players** with an average of **0.55 sessions per day** (as projected in the docs), that’s about 209,000 plays per day. At 1.25 KC burned each, **\~261,250 KC** burned daily. Over a year, that’s roughly **95 million KC burned from entry fees alone**. This aligns with the provided estimate (209k \* 365 \* 1.25 = \~95.4M).
    * In addition, **permanent item purchases** create burns. For example, cosmetic skins at 476 KC with 20% burn contribute significant deflation. The projections estimate about **39 million KC burned annually** via enhancements. Combined with entry fees, the ecosystem could burn on the order of **\~100+ million KC per year**.
    * At that rate, the supply deflates rapidly – however, note that not all 1 billion are in circulation initially (large portions are in reward pools, vesting, etc.). Even so, the model forecasts a strong deflationary trend, which can support token value if demand (from new players and staking incentives) remains high.


* **Staking Interest and Pool Longevity:** The staking interest pool of **175,000,000 KC** (part of the 250M rewards pool) is used to pay interest to stakers. The interest payout calculations provided show:
  * **Year 1:** \~200M KC staked on average, 10% effective APR → \~20M KC paid as interest.
  * The stake grows with new users (4% growth assumed) and interest compounding. By Year 5, payouts might be \~23M KC, and by Year 7 \~25M KC, nearly depleting the 175M if unchecked.
  * The model shows the pool lasting roughly **8 years** before running out, which is considered a sufficient runway. In fact, a highlight mentioned \~9 years under conservative growth, which is in the same ballpark. Our design choices (capping APR at 13%, requiring real player effort to reach that) ensure that not everyone gets the max rate, keeping the average APR around 10%. This matches the modeling assumptions and thus the interest payouts in the contract will align with the expected depletion schedule.
  * It’s worth noting that because the game burns are so high (\~100M/year), the circulating supply could decrease, meaning the relative inflation from interest (20M/year) is offset by even larger deflation. This could extend the economic sustainability beyond 8 years in terms of token value and reduce circulating supply, potentially increasing scarcity.

<figure><img src="../.gitbook/assets/The ecosystem is designed as a hybrid on-chain_off-chain architecture_ - visual selection (1).png" alt=""><figcaption></figcaption></figure>

* The **monthly reset** of points ensures no one can stockpile points infinitely to maintain high APR without continuing to play. This has been noted in the design; an implementation would involve either the backend or a scheduled on-chain call resetting the `points_month` fields in all profiles at the end of each month cycle.\

* **APR boost claim example:** Suppose a player had a stellar month with 60M in Degen Trader and 15M in SOLar Mission. The cap reduces Degen Trader’s count to 50M. Their total counting points = 50M + 15M = 65M. However, to qualify for Tier 6, they needed at least 20M from SOLar Mission (20% of 100M). They only have 15M there, so under the rules they actually would be limited to Tier 5 (because they failed the multi-game quota for Tier 6). The contract/algorithm would note that and likely treat their effective points as just enough for Tier 5. This nuance shows the complexity but it’s built into the multi-game rule. Next month, points go back to 0 so they have to play again to maintain or improve their tier.

