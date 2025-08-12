# Mutiplayer with Logic

### Crash Outcome & Cash-Out Logic

* If the player’s cash-out target (auto or manual) was **<= the crash multiplier**, it means the player successfully cashed out before the crash. The bet wins, and the points staked are multiplied by the target multiplier to calculate winnings. These **winnings are credited to the session in points** (the contract adds `bet_amount * multiplier` to the session’s point balance). For example, a 100-point bet cashed out at 2.5x yields 250 points (150 profit, 100 returned).
* If the player had **no cash-out or the target was higher than the crash**, the rocket crashes before they exited – the bet is lost and the staked points remain deducted. (In other words, failing to cash out in time means the player’s points for that bet are gone.)
* The contract allows one manual cash-out call per bet (players can cash out one bet and let the other ride, in a two-bet round, emulating the “split strategy”). The manual cash-out is recorded on-chain by updating the bet’s target multiplier at the moment of the user’s click, and the fairness is preserved by the VRF: because the actual crash outcome isn’t revealed until afterward, the player cannot know if they clicked “in time” except by luck. This logic aligns with the intended gameplay where manual timing is part of the challenge.

<figure><img src="../.gitbook/assets/_- visual selection (2).png" alt=""><figcaption></figcaption></figure>
