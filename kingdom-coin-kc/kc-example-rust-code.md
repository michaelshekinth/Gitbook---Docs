# KC Example Rust Code

{% hint style="info" %}
**Note : These are sample codes, But tested in the Dev environment.**&#x20;
{% endhint %}

```rust
// programs/kc_core/src/lib.rs
use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, Burn, Mint, Token, TokenAccount, Transfer,
};

declare_id!("KC11111111111111111111111111111111111111111"); // replace with your program id

// ---- constants ----
const SECONDS_PER_YEAR: i64 = 31_536_000; // 365d
const BPS_DENOMINATOR: u64 = 10_000;

// hard caps per-game per-month for points (these are "score units", not KC)
const MONTHLY_CAP_SOLAR: u64 = 50_000_000;
const MONTHLY_CAP_DEGEN: u64 = 50_000_000;

// entry fee: 12.5 KC = 12_500_000_000 in 9-decimals
const ENTRY_FEE_RAW: u64 = 12_500_000_000;
const ENTRY_BURN_BPS: u16 = 1000; // 10%
const UNSTAKE_BURN_BPS: u16 = 500; // 5%

#[program]
pub mod kc_core {
    use super::*;

    // ----------------------------------------------------------------
    // admin: one-time initialize
    // ----------------------------------------------------------------
    pub fn initialize(
        ctx: Context<Initialize>,
        base_apr_bps: u16,          // e.g. 800 for 8%
        max_apr_bps: u16,           // e.g. 1300 for 13%
        tier_thresholds: [u64; 6],  // [1M,5M,10M,25M,100M,100M+] points
        tier_boost_bps: [u16; 6],   // [0,100,200,300,400,500]
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        cfg.admin = ctx.accounts.admin.key();
        cfg.kc_mint = ctx.accounts.kc_mint.key();
        cfg.base_apr_bps = base_apr_bps;
        cfg.max_apr_bps = max_apr_bps;
        cfg.tier_thresholds = tier_thresholds;
        cfg.tier_boost_bps = tier_boost_bps;

        cfg.game_treasury = ctx.accounts.game_treasury.key();
        cfg.staking_vault = ctx.accounts.staking_vault.key();
        cfg.rewards_vault = ctx.accounts.rewards_vault.key();

        cfg.epoch_month = current_month_ts(ctx.accounts.clock.unix_timestamp);
        Ok(())
    }

    // ----------------------------------------------------------------
    // user: pay entry fee (burn 10% + send 90% to game_treasury)
    // callable by user (signer); amount is fixed ENTRY_FEE_RAW
    // ----------------------------------------------------------------
    pub fn pay_entry(ctx: Context<PayEntry>) -> Result<()> {
        // burn portion from user's ATA
        let burn_amount = mul_bps(ENTRY_FEE_RAW, ENTRY_BURN_BPS);
        let transfer_amount = ENTRY_FEE_RAW
            .checked_sub(burn_amount)
            .ok_or(ErrorCode::MathOverflow)?;

        // burn 10%
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.kc_mint.to_account_info(),
                    from: ctx.accounts.user_kc_ata.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            burn_amount,
        )?;

        // transfer 90% to game treasury
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_kc_ata.to_account_info(),
                    to: ctx.accounts.game_treasury.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            transfer_amount,
        )?;

        emit!(EntryPaid {
            user: ctx.accounts.user.key(),
            burn_amount,
            transferred: transfer_amount,
        });
        Ok(())
    }

    // ----------------------------------------------------------------
    // user: stake KC into program vault (custodial staking)
    // ----------------------------------------------------------------
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        // pull KC from user to staking_vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_kc_ata.to_account_info(),
                    to: ctx.accounts.staking_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        let stake = &mut ctx.accounts.user_stake;
        if stake.owner == Pubkey::default() {
            stake.owner = ctx.accounts.user.key();
            stake.kc_mint = ctx.accounts.kc_mint.key();
            stake.last_accrual_ts = ctx.accounts.clock.unix_timestamp;
            stake.epoch_month = current_month_ts(stake.last_accrual_ts);
        }

        // accrue before mutating
        accrue_rewards_internal(
            &mut *stake,
            &ctx.accounts.config,
            ctx.accounts.clock.unix_timestamp,
        )?;

        stake.staked_amount = stake
            .staked_amount
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;

        Ok(())
    }

    // ----------------------------------------------------------------
    // user: accrue & claim rewards to user's KC ATA (paid from rewards_vault)
    // ----------------------------------------------------------------
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let stake = &mut ctx.accounts.user_stake;

        accrue_rewards_internal(
            stake,
            &ctx.accounts.config,
            ctx.accounts.clock.unix_timestamp,
        )?;

        let amount = stake
            .pending_rewards
            .min(token_balance(&ctx.accounts.rewards_vault)?);

        if amount > 0 {
            stake.pending_rewards = stake
                .pending_rewards
                .checked_sub(amount)
                .ok_or(ErrorCode::MathOverflow)?;

            // transfer from rewards_vault (PDA-owned) to user
            let seeds = &[
                b"vault_auth",
                ctx.accounts.config.key().as_ref(),
                &[ctx.accounts.vault_auth_bump],
            ];
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.rewards_vault.to_account_info(),
                        to: ctx.accounts.user_kc_ata.to_account_info(),
                        authority: ctx.accounts.vault_authority.to_account_info(),
                    },
                    &[seeds],
                ),
                amount,
            )?;
        }

        emit!(RewardsClaimed {
            user: ctx.accounts.user.key(),
            amount,
        });
        Ok(())
    }

    // ----------------------------------------------------------------
    // user: unstake KC (burn 5% fee from vault, send 95% back to user)
    // ----------------------------------------------------------------
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);
        let stake = &mut ctx.accounts.user_stake;

        // accrue before mutating
        accrue_rewards_internal(
            stake,
            &ctx.accounts.config,
            ctx.accounts.clock.unix_timestamp,
        )?;

        require!(stake.staked_amount >= amount, ErrorCode::InsufficientStake);

        // compute burns & transfers
        let burn_amount = mul_bps(amount, UNSTAKE_BURN_BPS);
        let send_amount = amount
            .checked_sub(burn_amount)
            .ok_or(ErrorCode::MathOverflow)?;

        // burn from vault (program-controlled)
        let seeds = &[
            b"vault_auth",
            ctx.accounts.config.key().as_ref(),
            &[ctx.accounts.vault_auth_bump],
        ];
        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.kc_mint.to_account_info(),
                    from: ctx.accounts.staking_vault.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[seeds],
            ),
            burn_amount,
        )?;

        // transfer remainder to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staking_vault.to_account_info(),
                    to: ctx.accounts.user_kc_ata.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[seeds],
            ),
            send_amount,
        )?;

        stake.staked_amount = stake
            .staked_amount
            .checked_sub(amount)
            .ok_or(ErrorCode::MathOverflow)?;

        emit!(Unstaked {
            user: ctx.accounts.user.key(),
            burned: burn_amount,
            returned: send_amount,
        });
        Ok(())
    }

    // ----------------------------------------------------------------
    // games/oracle: update gameplay points for current month (per game)
    // authorized by admin or a pre-registered game/oracle authority
    // ----------------------------------------------------------------
    pub fn update_points(
        ctx: Context<UpdatePoints>,
        game: GameKind,
        add_points: u64,
    ) -> Result<()> {
        let stake = &mut ctx.accounts.user_stake;

        // rollover month if needed
        let now_month = current_month_ts(ctx.accounts.clock.unix_timestamp);
        if stake.epoch_month != now_month {
            stake.epoch_month = now_month;
            stake.points_solar = 0;
            stake.points_degen = 0;
        }

        match game {
            GameKind::Solar => {
                let newp = (stake.points_solar)
                    .saturating_add(add_points)
                    .min(MONTHLY_CAP_SOLAR);
                stake.points_solar = newp;
            }
            GameKind::Degen => {
                let newp = (stake.points_degen)
                    .saturating_add(add_points)
                    .min(MONTHLY_CAP_DEGEN);
                stake.points_degen = newp;
            }
        }

        // recompute tier from total points and multi-game requirement
        recompute_tier(stake, &ctx.accounts.config);

        emit!(PointsUpdated {
            user: ctx.accounts.user.key(),
            game,
            month: stake.epoch_month,
            solar: stake.points_solar,
            degen: stake.points_degen,
            tier: stake.current_tier,
        });
        Ok(())
    }

    // ----------------------------------------------------------------
    // admin: set new tiers/thresholds (governance-controlled)
    // ----------------------------------------------------------------
    pub fn set_tiers(
        ctx: Context<AdminOnly>,
        thresholds: [u64; 6],
        boosts_bps: [u16; 6],
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        cfg.tier_thresholds = thresholds;
        cfg.tier_boost_bps = boosts_bps;
        Ok(())
    }

    // admin: set base/max APR
    pub fn set_apr(ctx: Context<AdminOnly>, base_apr_bps: u16, max_apr_bps: u16) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        cfg.base_apr_bps = base_apr_bps;
        cfg.max_apr_bps = max_apr_bps;
        Ok(())
    }
}

// ========================= helpers =========================

fn accrue_rewards_internal(
    stake: &mut UserStake,
    cfg: &Config,
    now_ts: i64,
) -> Result<()> {
    // time delta
    let dt = now_ts
        .checked_sub(stake.last_accrual_ts)
        .ok_or(ErrorCode::MathOverflow)?;
    if dt <= 0 {
        return Ok(());
    }

    // base + tier boost, capped
    let mut apr_bps = cfg.base_apr_bps as u64 + (stake.current_boost_bps as u64);
    if apr_bps > cfg.max_apr_bps as u64 {
        apr_bps = cfg.max_apr_bps as u64;
    }

    // rewards = stake * apr_bps/10000 * dt / seconds_per_year
    // do in u128 to avoid overflow
    let stake_amt = stake.staked_amount as u128;
    let apr_bps_u = apr_bps as u128;
    let dt_u = dt as u128;
    let reward = stake_amt
        .saturating_mul(apr_bps_u)
        .saturating_mul(dt_u)
        / (BPS_DENOMINATOR as u128)
        / (SECONDS_PER_YEAR as u128);

    let reward_u64 = u64::try_from(reward).map_err(|_| ErrorCode::MathOverflow)?;
    stake.pending_rewards = stake
        .pending_rewards
        .checked_add(reward_u64)
        .ok_or(ErrorCode::MathOverflow)?;

    stake.last_accrual_ts = now_ts;
    Ok(())
}

fn recompute_tier(stake: &mut UserStake, cfg: &Config) {
    let total = stake.points_solar.saturating_add(stake.points_degen);

    // multi-game requirement for top tiers (4–6): need >=20% from each game
    let multi_ok = stake.points_solar >= total / 5 && stake.points_degen >= total / 5;

    let mut tier = 0usize;
    for (i, th) in cfg.tier_thresholds.iter().enumerate() {
        if total >= *th {
            tier = i;
        }
    }

    // if tier >= 3 (0-based: 3..=5) enforce multi-game requirement
    if tier >= 3 && !multi_ok {
        // drop to tier 3-1 = 2 if requirement not met
        // (customize as you like)
        tier = 2;
    }

    stake.current_tier = tier as u8;
    stake.current_boost_bps = cfg.tier_boost_bps[tier];
}

fn mul_bps(amount: u64, bps: u16) -> u64 {
    ((amount as u128) * (bps as u128) / (BPS_DENOMINATOR as u128)) as u64
}

fn current_month_ts(unix_ts: i64) -> i64 {
    // rounds down to (approx) month buckets using 30-day months; cheap for on-chain
    unix_ts / (60 * 60 * 24 * 30)
}

fn token_balance(ata: &Account<TokenAccount>) -> Result<u64> {
    Ok(ata.amount)
}

// ========================= accounts =========================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub kc_mint: Account<'info, Mint>,

    /// CHECK: PDA that owns staking & rewards vaults
    #[account(
        seeds = [b"vault_auth", config.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    // program-owned token accounts (precreated) with owner = vault_authority
    #[account(mut, constraint = staking_vault.mint == kc_mint.key())]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(mut, constraint = rewards_vault.mint == kc_mint.key())]
    pub rewards_vault: Account<'info, TokenAccount>,

    // KC treasury (game prize/dev pool) – typically a multisig-owned ATA
    #[account(mut, constraint = game_treasury.mint == kc_mint.key())]
    pub game_treasury: Account<'info, TokenAccount>,

    #[account(init, payer = admin, space = 8 + Config::SIZE)]
    pub config: Account<'info, Config>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct PayEntry<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub kc_mint: Account<'info, Mint>,

    #[account(mut, has_one = kc_mint)]
    pub config: Account<'info, Config>,

    #[account(mut, constraint = user_kc_ata.mint == kc_mint.key(), constraint = user_kc_ata.owner == user.key())]
    pub user_kc_ata: Account<'info, TokenAccount>,

    #[account(mut, address = config.game_treasury)]
    pub game_treasury: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub kc_mint: Account<'info, Mint>,

    #[account(mut, has_one = kc_mint)]
    pub config: Account<'info, Config>,

    #[account(mut, constraint = user_kc_ata.mint == kc_mint.key(), constraint = user_kc_ata.owner == user.key())]
    pub user_kc_ata: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for vaults
    #[account(
        seeds = [b"vault_auth", config.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut, address = config.staking_vault)]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserStake::SIZE,
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,

    // bump needed for signed CPI on vault ops later
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub kc_mint: Account<'info, Mint>,

    #[account(mut, has_one = kc_mint)]
    pub config: Account<'info, Config>,

    #[account(mut, constraint = user_kc_ata.mint == kc_mint.key(), constraint = user_kc_ata.owner == user.key())]
    pub user_kc_ata: Account<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(
        seeds = [b"vault_auth", config.key().as_ref()],
        bump = vault_auth_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub vault_auth_bump: u8,

    #[account(mut, address = config.rewards_vault)]
    pub rewards_vault: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"stake", user.key().as_ref()], bump)]
    pub user_stake: Account<'info, UserStake>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub kc_mint: Account<'info, Mint>,

    #[account(mut, has_one = kc_mint)]
    pub config: Account<'info, Config>,

    #[account(mut, constraint = user_kc_ata.mint == kc_mint.key(), constraint = user_kc_ata.owner == user.key())]
    pub user_kc_ata: Account<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(
        seeds = [b"vault_auth", config.key().as_ref()],
        bump = vault_auth_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub vault_auth_bump: u8,

    #[account(mut, address = config.staking_vault)]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(mut, seeds = [b"stake", user.key().as_ref()], bump)]
    pub user_stake: Account<'info, UserStake>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct UpdatePoints<'info> {
    // signer must be admin OR a registered oracle/game authority off-chain (you can expand this with a registry)
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(has_one = kc_mint)]
    pub config: Account<'info, Config>,

    pub kc_mint: Account<'info, Mint>,

    #[account(mut, seeds = [b"stake", user.key().as_ref()], bump)]
    pub user_stake: Account<'info, UserStake>,

    /// CHECK: user pubkey (doesn't need to sign)
    pub user: UncheckedAccount<'info>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    pub admin: Signer<'info>,
    #[account(mut, constraint = config.admin == admin.key())]
    pub config: Account<'info, Config>,
}

// ========================= data =========================

#[account]
pub struct Config {
    pub admin: Pubkey,
    pub kc_mint: Pubkey,

    pub staking_vault: Pubkey,
    pub rewards_vault: Pubkey,
    pub game_treasury: Pubkey,

    pub base_apr_bps: u16,   // e.g. 800
    pub max_apr_bps: u16,    // e.g. 1300

    pub tier_thresholds: [u64; 6],
    pub tier_boost_bps: [u16; 6],

    pub epoch_month: i64, // month bucket
}
impl Config {
    pub const SIZE: usize = 32 + 32 + 32 + 32 + 32 + 2 + 2 + (8 * 6) + (2 * 6) + 8;
}

#[account]
pub struct UserStake {
    pub owner: Pubkey,
    pub kc_mint: Pubkey,

    pub staked_amount: u64,
    pub pending_rewards: u64,

    pub last_accrual_ts: i64,
    pub epoch_month: i64,

    // points this month (per game)
    pub points_solar: u64,
    pub points_degen: u64,

    // derived tier and boost bps
    pub current_tier: u8,      // 0..=5
    pub current_boost_bps: u16 // e.g., 0..=500
}
impl UserStake {
    pub const SIZE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 2;
}

// ========================= events =========================
#[event]
pub struct EntryPaid {
    pub user: Pubkey,
    pub burn_amount: u64,
    pub transferred: u64,
}

#[event]
pub struct RewardsClaimed {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Unstaked {
    pub user: Pubkey,
    pub burned: u64,
    pub returned: u64,
}

#[event]
pub struct PointsUpdated {
    pub user: Pubkey,
    pub game: GameKind,
    pub month: i64,
    pub solar: u64,
    pub degen: u64,
    pub tier: u8,
}

// ========================= types & errors =========================
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameKind {
    Solar,
    Degen,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
}

```

**The code explanation is given in the next page.**
