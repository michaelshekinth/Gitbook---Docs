# Sample Contract

**Note: This is a smaple smart contract - It do not consists of VRF,  Roolover Logics, No Account Sizings, Etc... This contract is the check the core functions**

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer, Burn};

// Constants for game mechanics and tokenomics
const ENTRY_FEE_POINTS: u64 = 2000;                // Points granted per session (cost 12.5 KC)
const ENTRY_FEE_KC: u64 = 12_500_000_000;          // 12.5 KC in smallest units (KC has 9 decimals)
const BURN_FEE_KC: u64 = 1_250_000_000;            // 1.25 KC (10% of entry) in smallest units
const MIN_BET_POINTS: u64 = 10;
const MAX_BET_POINTS: u64 = 1000;
const MAX_POINTS_PER_MONTH: u64 = 50_000_000;      // max points per month from this game
const MULTIPLIER_SCALE: u32 = 100;                // multiplier precision (100 = 2 decimal places)
const MAX_MULTIPLIER: u32 = 1000000;              // 1000000 = 10000.00x max multiplier

declare_id!("GameXXXXXXXXXXXXXXXXXXXXXXXXXXXX");  // Placeholder program ID

#[program]
pub mod solar_mission_crash {
    use super::*;

    /// One-time initializer for the game configuration (called by admin)
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.kc_mint = ctx.accounts.kc_mint.key();
        config.treasury_vault = ctx.accounts.treasury_vault.key();
        config.authority = ctx.accounts.admin.key();
        config.bump = *ctx.bumps.get("config").unwrap();
        Ok(())
    }

    /// Start a new game session for the player, charging the entry fee.
    pub fn start_session(ctx: Context<StartSession>) -> Result<()> {
        let config = &ctx.accounts.config;
        let session = &mut ctx.accounts.session;
        let profile = &mut ctx.accounts.profile;
        let user_token_account = &ctx.accounts.user_token;

        // Ensure no active session is currently running for this user
        require!(!session.active, ErrorCode::SessionAlreadyActive);

        // Burn 1.25 KC (10% of fee) from the user's token account
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.kc_mint.to_account_info(),
                    from: user_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            BURN_FEE_KC,
        )?;
        // Transfer 11.25 KC (remaining fee) from user to the treasury vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: user_token_account.to_account_info(),
                    to: ctx.accounts.treasury_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            ENTRY_FEE_KC - BURN_FEE_KC,
        )?;

        // Initialize session state for a new game
        session.user = ctx.accounts.user.key();
        session.active = true;
        session.round_active = false;
        session.points = ENTRY_FEE_POINTS;     // start with 2000 points
        session.bet1 = None;
        session.bet2 = None;

        // Update user profile stats
        profile.user = ctx.accounts.user.key();
        profile.session_count = profile.session_count.checked_add(1).unwrap();
        // (points_this_month is updated on end_session; best_session_points updated on end if applicable)

        Ok(())
    }

    /// Place one or two bets to begin a new round.
    /// `bet1_amount` is required, `bet2_amount` is optional (pass None if only one bet).
    /// Auto cash-out multipliers can be provided (as scaled integers, e.g., 250 for 2.50x).
    pub fn place_bets(
        ctx: Context<PlaceBets>,
        bet1_amount: u64,
        bet1_auto_multiplier: Option<u32>,
        bet2_amount: Option<u64>,
        bet2_auto_multiplier: Option<u32>,
    ) -> Result<()> {
        let session = &mut ctx.accounts.session;
        require!(session.active, ErrorCode::SessionNotActive);
        require!(!session.round_active, ErrorCode::RoundAlreadyInProgress);

        // Validate bet1
        require!(bet1_amount >= MIN_BET_POINTS && bet1_amount <= MAX_BET_POINTS, ErrorCode::InvalidBetAmount);
        let mut total_bet = bet1_amount;
        // Validate bet2 if provided
        if let Some(b2) = bet2_amount {
            require!(b2 >= MIN_BET_POINTS && b2 <= MAX_BET_POINTS, ErrorCode::InvalidBetAmount);
            total_bet = total_bet.checked_add(b2).ok_or(ErrorCode::InvalidBetAmount)?;
        }
        require!(total_bet <= session.points, ErrorCode::InsufficientPoints);

        // Deduct the bet amounts from session points (lock for the round)
        session.points = session.points.checked_sub(total_bet).unwrap();

        // Record Bet 1
        session.bet1 = Some(Bet {
            amount: bet1_amount,
            cashout_multiplier: bet1_auto_multiplier, // auto cash-out target if any
            cashed_out: false,
        });
        // Record Bet 2 if exists
        session.bet2 = if let Some(b2) = bet2_amount {
            Some(Bet {
                amount: b2,
                cashout_multiplier: bet2_auto_multiplier,
                cashed_out: false,
            })
        } else {
            None
        };

        session.round_active = true;

        // ** Trigger VRF for random crash point here **
        // (In practice, call out to Switchboard VRF oracle. The VRF's callback will invoke resolve_round.)

        Ok(())
    }

    /// Manually cash out a bet early (before the round ends).
    /// `bet_index` selects which bet (1 or 2) to cash out, and `target_multiplier` is the multiplier (scaled by 100) at the time of cash-out.
    pub fn cash_out(ctx: Context<CashOut>, bet_index: u8, target_multiplier: u32) -> Result<()> {
        let session = &mut ctx.accounts.session;
        require!(session.active && session.round_active, ErrorCode::NoActiveRound);
        require!(target_multiplier >= MULTIPLIER_SCALE && target_multiplier <= MAX_MULTIPLIER, ErrorCode::InvalidMultiplier);

        // Select the bet to cash out (bet1 or bet2)
        let bet_opt = if bet_index == 1 {
            session.bet1.as_mut()
        } else if bet_index == 2 {
            session.bet2.as_mut()
        } else {
            return Err(ErrorCode::InvalidBetIndex.into());
        };
        let bet = bet_opt.ok_or(ErrorCode::BetNotFound)?;

        require!(!bet.cashed_out, ErrorCode::BetAlreadyCashed);
        // Update the bet's cash-out multiplier. 
        // If no auto was set or if the manual target is lower (earlier) than an existing auto, use the new target.
        if bet.cashout_multiplier.is_none() || target_multiplier < bet.cashout_multiplier.unwrap() {
            bet.cashout_multiplier = Some(target_multiplier);
        }
        bet.cashed_out = true;  // mark that player has cashed out this bet (from their perspective)

        // (No payout yet; the actual outcome will be handled in resolve_round)

        Ok(())
    }

    /// Resolve the round after receiving the random crash point. This will calculate outcomes for bets.
    /// (In a real deployment, this should only be called by the VRF oracle or program, not an arbitrary user.)
    pub fn resolve_round(ctx: Context<ResolveRound>) -> Result<()> {
        let session = &mut ctx.accounts.session;
        require!(session.active && session.round_active, ErrorCode::NoActiveRound);

        // In practice, the crash_multiplier comes from a VRF proof. Here we simulate it for demonstration:
        let clock = Clock::get().unwrap();
        let random_seed = clock.unix_timestamp as u64;
        // Derive a pseudo-random crash multiplier in [1.00, 10000.00]x range
        let mut crash_multiplier: u32 = ((random_seed % (MAX_MULTIPLIER as u64 - (MULTIPLIER_SCALE as u64))) as u32) + MULTIPLIER_SCALE;
        if crash_multiplier > MAX_MULTIPLIER {
            crash_multiplier = MAX_MULTIPLIER;
        }

        // Resolve Bet 1 outcome
        if let Some(bet) = session.bet1 {
            process_betOutcome(session, bet, crash_multiplier)?;
        }
        // Resolve Bet 2 outcome
        if let Some(bet) = session.bet2 {
            process_betOutcome(session, bet, crash_multiplier)?;
        }

        // Clear bets and mark round as over
        session.bet1 = None;
        session.bet2 = None;
        session.round_active = false;

        // If the player has no points left, we can mark the session as ended (busted)
        if session.points == 0 {
            session.active = false;
            // (Optionally, auto-call end_session here. In this implementation, the user can call end_session separately, but it's marked inactive so they can start a new session if they want.)
        }

        Ok(())
    }

    /// End the session and record the results to the user's profile.
    pub fn end_session(ctx: Context<EndSession>) -> Result<()> {
        let session = &mut ctx.accounts.session;
        let profile = &mut ctx.accounts.profile;
        require!(session.active && !session.round_active, ErrorCode::SessionNotActive);

        let final_points = session.points;
        // Add session points to monthly cumulative score, respecting the monthly cap
        let current = profile.points_this_month;
        let new_total = current.saturating_add(final_points);
        profile.points_this_month = if new_total > MAX_POINTS_PER_MONTH {
            MAX_POINTS_PER_MONTH
        } else {
            new_total
        };
        // Update best session score if this session is the highest
        if final_points > profile.best_session_points {
            profile.best_session_points = final_points;
        }

        // Mark session as ended (deactivate it)
        session.active = false;
        session.points = 0;
        session.bet1 = None;
        session.bet2 = None;
        session.round_active = false;

        Ok(())
    }
}

/// Internal helper: process outcome for a single bet given the crash multiplier.
fn process_betOutcome(session: &mut Account<Session>, bet: Bet, crash_multiplier: u32) -> Result<()> {
    let amount = bet.amount;
    if amount == 0 {
        return Ok(()); // no bet to process
    }
    if let Some(target) = bet.cashout_multiplier {
        if target <= crash_multiplier {
            // Player cashed out successfully at target multiplier
            let payout = (amount as u128)
                .checked_mul(target as u128).unwrap()
                .checked_div(MULTIPLIER_SCALE as u128).unwrap() as u64;
            session.points = session.points.checked_add(payout).unwrap();
        } 
        // else: target > crash_multiplier => crash happened before cashout, bet is lost (already deducted)
    } else {
        // No cashout set -> player rode indefinitely, so if crash_multiplier > 0 (always true here), they get nothing (bet lost).
    }
    Ok(())
}

// Account structures and Anchor account definitions

#[account]
pub struct GameConfig {
    pub kc_mint: Pubkey,        // KC token mint address
    pub treasury_vault: Pubkey, // Token account for holding collected fees (for prizes/treasury)
    pub authority: Pubkey,      // Admin authority (could be used for future admin actions like resetting points or distributing prizes)
    pub bump: u8,
}

#[account]
pub struct Session {
    pub user: Pubkey,
    pub active: bool,
    pub round_active: bool,
    pub points: u64,
    pub bet1: Option<Bet>,      // Details of bet 1 (if any) for current round
    pub bet2: Option<Bet>,      // Details of bet 2 (if any)
}

#[account]
pub struct UserProfile {
    pub user: Pubkey,
    pub points_this_month: u64,
    pub best_session_points: u64,
    pub session_count: u32,
    // (Additional fields like last_reset_timestamp or country could be added for more features)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Bet {
    pub amount: u64,
    pub cashout_multiplier: Option<u32>,  // target multiplier (scaled by 100) for auto or manual cash-out
    pub cashed_out: bool,                // whether the player initiated a cash-out for this bet
}

// Accounts context definitions for each instruction

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, seeds = [b"config"], bump, payer = admin, space = 8 + 32*3 + 1)]
    pub config: Account<'info, GameConfig>,
    pub kc_mint: Account<'info, Mint>,
    #[account(init, associated_token::mint = kc_mint, associated_token::authority = config, payer = admin)]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct StartSession<'info> {
    #[account(mut, seeds=[b"config"], bump = config.bump, has_one = kc_mint, has_one = treasury_vault)]
    pub config: Account<'info, GameConfig>,
    #[account(init_if_needed, seeds=[b"session", user.key().as_ref()], bump, payer = user, space = 8 + 32 + 1 + 1 + 8 + (/*Bet struct size*/ 16 * 2))]
    pub session: Account<'info, Session>,
    #[account(init_if_needed, seeds=[b"profile", user.key().as_ref()], bump, payer = user, space = 8 + 32 + 8 + 8 + 4)]
    pub profile: Account<'info, UserProfile>,
    pub kc_mint: Account<'info, Mint>,
    #[account(mut, token::mint = kc_mint, token::authority = user)]
    pub user_token: Account<'info, TokenAccount>,    // player's KC token account (to pay fee)
    #[account(mut, token::mint = kc_mint, token::authority = config)]
    pub treasury_vault: Account<'info, TokenAccount>,// game treasury vault to collect fees
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct PlaceBets<'info> {
    #[account(mut, has_one = user)]
    pub session: Account<'info, Session>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct CashOut<'info> {
    #[account(mut, has_one = user)]
    pub session: Account<'info, Session>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResolveRound<'info> {
    #[account(mut)]
    pub session: Account<'info, Session>,
    // In a real VRF callback, we'd include the VRF account and verify the randomness here.
    // No signer required if called by the oracle program.
}

#[derive(Accounts)]
pub struct EndSession<'info> {
    #[account(mut, has_one = user)]
    pub session: Account<'info, Session>,
    #[account(mut, has_one = user)]
    pub profile: Account<'info, UserProfile>,
    pub user: Signer<'info>,
}

// Custom error codes for clear error messages
#[error_code]
pub enum ErrorCode {
    #[msg("Session is already active for this user.")]
    SessionAlreadyActive,
    #[msg("No active session or incorrect session state.")]
    SessionNotActive,
    #[msg("A round is already in progress.")]
    RoundAlreadyInProgress,
    #[msg("Insufficient points to place the bet(s).")]
    InsufficientPoints,
    #[msg("Bet amount must be between 10 and 1000 points.")]
    InvalidBetAmount,
    #[msg("No active round to cash out from.")]
    NoActiveRound,
    #[msg("Invalid bet index (must be 1 or 2).")]
    InvalidBetIndex,
    #[msg("Bet not found.")]
    BetNotFound,
    #[msg("This bet has already been cashed out.")]
    BetAlreadyCashed,
    #[msg("Cash-out multiplier is out of valid range.")]
    InvalidMultiplier,
}

```
