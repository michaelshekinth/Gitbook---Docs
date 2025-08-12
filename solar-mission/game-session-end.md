# Game Session End

### **Points Conversion to Staking Rewards**

When a gaming session concludes, any remaining points (or zero if the player has busted) are converted on a one-to-one basis into **staking points** which are then attributed to the player’s profile. These points are managed via a **UserProfile PDA** that is created per player to facilitate the accumulation of these points on a monthly basis. Importantly, there is a stipulation that a maximum of **50,000,000 points per month** can be credited from a single game, ensuring that any points garnered above this threshold are disregarded. This rule is critical in maintaining a balanced tokenomics structure as it prevents any single player or game from skewing the Annual Percentage Rate (APR) advantages disproportionately.

These staking points are instrumental for contributing to a player’s staking APR boost tier. According to the governing tokenomics, while the foundational staking APR stands at 8%, players have the potential to elevate this percentage by up to an additional 5%, culminating in a maximum APR of 13%. This enhanced APR can be achieved by reaching specific high-score benchmarks. For instance, amassing 100 million points in any given month, spanning across various game types, enables the player to attain the apex APR boost of an extra 5%. However, it’s imperative to note that the contract ensures that only up to 50 million of these points may originate from a single game like SOLar Mission, thus promoting diverse participation across multiple gaming formats.

The profile account meticulously logs the cumulative points, and this data can be leveraged by either staking programs or off-chain processes to adjust and fine-tune APR for each user, providing a tailored incentive structure.

