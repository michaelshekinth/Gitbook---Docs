use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;
use std::collections::HashMap;

declare_id!("DegenTraderGameProgram11111111111111111111");

#[program]
pub mod degen_trader {
    use super::*;

    pub fn initialize_game(ctx: Context<InitializeGame>) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        game_state.authority = ctx.accounts.authority.key();
        game_state.total_players = 0;
        game_state.current_day = 0;
        game_state.is_active = true;
        
        // Initialize coin prices (in cents to avoid decimals)
        game_state.coin_prices = vec![
            CoinPrice { name: "DOGE".to_string(), price: 10 },      // $0.10
            CoinPrice { name: "SHIB".to_string(), price: 1 },       // $0.01
            CoinPrice { name: "PEPE".to_string(), price: 1 },       // $0.01
            CoinPrice { name: "BONK".to_string(), price: 1 },       // $0.01
            CoinPrice { name: "WIF".to_string(), price: 50 },       // $0.50
            CoinPrice { name: "FLOKI".to_string(), price: 5 },      // $0.05
        ];
        
        Ok(())
    }

    pub fn create_player(ctx: Context<CreatePlayer>) -> Result<()> {
        let player_session = &mut ctx.accounts.player_session;
        let game_state = &mut ctx.accounts.game_state;
        
        player_session.player = ctx.accounts.player.key();
        player_session.cash = 1000_00; // $1000 in cents
        player_session.location = Location::Bedroom;
        player_session.day = 0;
        player_session.net_worth = 1000_00;
        player_session.portfolio = HashMap::new();
        player_session.is_active = true;
        
        game_state.total_players += 1;
        
        Ok(())
    }

    pub fn next_day(
        ctx: Context<NextDay>,
        trades: Vec<TradeOrder>
    ) -> Result<()> {
        let player_session = &mut ctx.accounts.player_session;
        let game_state = &mut ctx.accounts.game_state;
        
        require!(player_session.is_active, GameError::PlayerNotActive);
        require!(game_state.is_active, GameError::GameNotActive);
        
        // Generate random event for the day
        let random_event = generate_random_event(
            &ctx.accounts.player.key(),
            game_state.current_day,
            &ctx.accounts.recent_blockhashes.data.borrow()
        )?;
        
        // Update coin prices based on random event
        update_coin_prices(&mut game_state.coin_prices, &random_event)?;
        
        // Process player trades
        process_trades(player_session, &game_state.coin_prices, trades)?;
        
        // Update player's net worth
        calculate_net_worth(player_session, &game_state.coin_prices)?;
        
        // Check for location progression
        check_location_progression(player_session)?;
        
        // Increment day
        player_session.day += 1;
        if player_session.day > game_state.current_day {
            game_state.current_day = player_session.day;
        }
        
        emit!(DayCompleted {
            player: ctx.accounts.player.key(),
            day: player_session.day,
            event: random_event,
            net_worth: player_session.net_worth,
            location: player_session.location.clone(),
        });
        
        Ok(())
    }

    pub fn get_player_stats(ctx: Context<GetPlayerStats>) -> Result<PlayerStats> {
        let player_session = &ctx.accounts.player_session;
        
        Ok(PlayerStats {
            cash: player_session.cash,
            net_worth: player_session.net_worth,
            location: player_session.location.clone(),
            day: player_session.day,
            portfolio_value: calculate_portfolio_value(
                &player_session.portfolio,
                &ctx.accounts.game_state.coin_prices
            )?,
        })
    }
}

// Helper functions
fn generate_random_event(
    player_key: &Pubkey,
    day: u64,
    recent_blockhashes: &[u8]
) -> Result<RandomEvent> {
    let seed = [
        player_key.as_ref(),
        &day.to_le_bytes(),
        &recent_blockhashes[0..8]
    ].concat();
    
    let hash_result = hash(&seed);
    let random_value = u64::from_le_bytes([
        hash_result.0[0], hash_result.0[1], hash_result.0[2], hash_result.0[3],
        hash_result.0[4], hash_result.0[5], hash_result.0[6], hash_result.0[7],
    ]);
    
    let event_type = match random_value % 100 {
        0..=19 => EventType::ElonTweet,
        20..=29 => EventType::RugPull,
        30..=34 => EventType::TaxRaid,
        35..=44 => EventType::WhaleMovement,
        45..=54 => EventType::ExchangeListing,
        55..=64 => EventType::RegulationNews,
        65..=74 => EventType::CelebEndorsement,
        75..=84 => EventType::MarketCrash,
        85..=94 => EventType::BullRun,
        _ => EventType::Normal,
    };
    
    let affected_coin_index = (random_value >> 8) % 6;
    let multiplier = match event_type {
        EventType::ElonTweet => 500, // 5x multiplier
        EventType::RugPull => 0,     // Price goes to 0
        EventType::TaxRaid => 50,    // 50% drop
        EventType::WhaleMovement => 200, // 2x
        EventType::ExchangeListing => 300, // 3x
        EventType::RegulationNews => 70,   // 30% drop
        EventType::CelebEndorsement => 250, // 2.5x
        EventType::MarketCrash => 30,      // 70% drop
        EventType::BullRun => 150,         // 1.5x
        EventType::Normal => 100,          // No change
    };
    
    Ok(RandomEvent {
        event_type,
        affected_coin_index: affected_coin_index as u8,
        price_multiplier: multiplier,
        description: get_event_description(&event_type),
    })
}

fn update_coin_prices(coin_prices: &mut Vec<CoinPrice>, event: &RandomEvent) -> Result<()> {
    if let Some(coin) = coin_prices.get_mut(event.affected_coin_index as usize) {
        if event.event_type == EventType::RugPull {
            coin.price = 0;
        } else {
            coin.price = (coin.price as u64 * event.price_multiplier as u64 / 100) as u32;
            // Ensure minimum price of 1 cent
            if coin.price == 0 {
                coin.price = 1;
            }
        }
    }
    
    // Add some random volatility to other coins
    for (i, coin) in coin_prices.iter_mut().enumerate() {
        if i != event.affected_coin_index as usize {
            let volatility = ((i as u32 + event.price_multiplier) % 20) as i32 - 10; // -10% to +10%
            let new_price = coin.price as i32 + (coin.price as i32 * volatility / 100);
            coin.price = std::cmp::max(1, new_price) as u32;
        }
    }
    
    Ok(())
}

fn process_trades(
    player_session: &mut PlayerSession,
    coin_prices: &[CoinPrice],
    trades: Vec<TradeOrder>
) -> Result<()> {
    for trade in trades {
        let coin_price = coin_prices.iter()
            .find(|c| c.name == trade.coin_name)
            .ok_or(GameError::InvalidCoin)?;
        
        let trading_fee = calculate_trading_fee(player_session.location.clone());
        
        match trade.action {
            TradeAction::Buy => {
                let total_cost = trade.quantity as u64 * coin_price.price as u64;
                let fee = total_cost * trading_fee / 10000; // Fee in basis points
                let total_with_fee = total_cost + fee;
                
                require!(player_session.cash >= total_with_fee, GameError::InsufficientFunds);
                
                player_session.cash -= total_with_fee;
                *player_session.portfolio.entry(trade.coin_name).or_insert(0) += trade.quantity;
            },
            TradeAction::Sell => {
                let current_holdings = *player_session.portfolio.get(&trade.coin_name).unwrap_or(&0);
                require!(current_holdings >= trade.quantity, GameError::InsufficientHoldings);
                
                let total_value = trade.quantity as u64 * coin_price.price as u64;
                let fee = total_value * trading_fee / 10000;
                let total_after_fee = total_value - fee;
                
                player_session.cash += total_after_fee;
                *player_session.portfolio.entry(trade.coin_name).or_insert(0) -= trade.quantity;
                
                // Remove entry if quantity becomes 0
                if player_session.portfolio[&trade.coin_name] == 0 {
                    player_session.portfolio.remove(&trade.coin_name);
                }
            }
        }
    }
    
    Ok(())
}

fn calculate_trading_fee(location: Location) -> u64 {
    match location {
        Location::Bedroom => 200,    // 2% fee
        Location::Garage => 150,     // 1.5% fee
        Location::Office => 100,     // 1% fee
        Location::Yacht => 50,       // 0.5% fee
        Location::Penthouse => 25,   // 0.25% fee
    }
}

fn calculate_net_worth(
    player_session: &mut PlayerSession,
    coin_prices: &[CoinPrice]
) -> Result<()> {
    let portfolio_value = calculate_portfolio_value(&player_session.portfolio, coin_prices)?;
    player_session.net_worth = player_session.cash + portfolio_value;
    Ok(())
}

fn calculate_portfolio_value(
    portfolio: &HashMap<String, u32>,
    coin_prices: &[CoinPrice]
) -> Result<u64> {
    let mut total_value = 0u64;
    
    for (coin_name, quantity) in portfolio {
        if let Some(coin_price) = coin_prices.iter().find(|c| &c.name == coin_name) {
            total_value += *quantity as u64 * coin_price.price as u64;
        }
    }
    
    Ok(total_value)
}

fn check_location_progression(player_session: &mut PlayerSession) -> Result<()> {
    let new_location = match player_session.net_worth {
        0..=999_99 => Location::Bedroom,
        1000_00..=9999_99 => Location::Garage,
        10000_00..=99999_99 => Location::Office,
        100000_00..=999999_99 => Location::Yacht,
        _ => Location::Penthouse,
    };
    
    if new_location != player_session.location {
        player_session.location = new_location;
    }
    
    Ok(())
}

fn get_event_description(event_type: &EventType) -> String {
    match event_type {
        EventType::ElonTweet => "Elon tweeted about meme coins! 🚀".to_string(),
        EventType::RugPull => "Major rug pull detected! 💸".to_string(),
        EventType::TaxRaid => "Tax authorities raid crypto exchange! 🏛️".to_string(),
        EventType::WhaleMovement => "Crypto whale makes massive move! 🐋".to_string(),
        EventType::ExchangeListing => "New exchange listing announced! 📈".to_string(),
        EventType::RegulationNews => "New crypto regulations announced! ⚖️".to_string(),
        EventType::CelebEndorsement => "Celebrity endorses crypto! ⭐".to_string(),
        EventType::MarketCrash => "Market crash hits crypto! 📉".to_string(),
        EventType::BullRun => "Bull run incoming! 🐂".to_string(),
        EventType::Normal => "Normal trading day 📊".to_string(),
    }
}

// Account structures
#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GameState::INIT_SPACE,
        seeds = [b"game_state"],
        bump
    )]
    pub game_state: Account<'info, GameState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePlayer<'info> {
    #[account(
        init,
        payer = player,
        space = 8 + PlayerSession::INIT_SPACE,
        seeds = [b"player_session", player.key().as_ref()],
        bump
    )]
    pub player_session: Account<'info, PlayerSession>,
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct NextDay<'info> {
    #[account(mut)]
    pub player_session: Account<'info, PlayerSession>,
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    pub player: Signer<'info>,
    /// CHECK: This account is used for randomness
    pub recent_blockhashes: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GetPlayerStats<'info> {
    pub player_session: Account<'info, PlayerSession>,
    pub game_state: Account<'info, GameState>,
    pub player: Signer<'info>,
}

// Data structures
#[account]
pub struct GameState {
    pub authority: Pubkey,
    pub total_players: u64,
    pub current_day: u64,
    pub is_active: bool,
    pub coin_prices: Vec<CoinPrice>,
}

impl GameState {
    pub const INIT_SPACE: usize = 32 + 8 + 8 + 1 + 4 + (6 * (4 + 32 + 4)); // Approximate space
}

#[account]
pub struct PlayerSession {
    pub player: Pubkey,
    pub cash: u64,
    pub location: Location,
    pub day: u64,
    pub net_worth: u64,
    pub portfolio: HashMap<String, u32>,
    pub is_active: bool,
}

impl PlayerSession {
    pub const INIT_SPACE: usize = 32 + 8 + 1 + 8 + 8 + 200 + 1; // Approximate space for HashMap
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CoinPrice {
    pub name: String,
    pub price: u32, // Price in cents
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum Location {
    Bedroom,
    Garage,
    Office,
    Yacht,
    Penthouse,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TradeOrder {
    pub coin_name: String,
    pub quantity: u32,
    pub action: TradeAction,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum TradeAction {
    Buy,
    Sell,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RandomEvent {
    pub event_type: EventType,
    pub affected_coin_index: u8,
    pub price_multiplier: u32,
    pub description: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EventType {
    ElonTweet,
    RugPull,
    TaxRaid,
    WhaleMovement,
    ExchangeListing,
    RegulationNews,
    CelebEndorsement,
    MarketCrash,
    BullRun,
    Normal,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PlayerStats {
    pub cash: u64,
    pub net_worth: u64,
    pub location: Location,
    pub day: u64,
    pub portfolio_value: u64,
}

// Events
#[event]
pub struct DayCompleted {
    pub player: Pubkey,
    pub day: u64,
    pub event: RandomEvent,
    pub net_worth: u64,
    pub location: Location,
}

// Errors
#[error_code]
pub enum GameError {
    #[msg("Player is not active")]
    PlayerNotActive,
    #[msg("Game is not active")]
    GameNotActive,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Insufficient holdings")]
    InsufficientHoldings,
    #[msg("Invalid coin")]
    InvalidCoin,
    #[msg("Invalid trade amount")]
    InvalidTradeAmount,
}