use anchor_lang::prelude::*;
use psyfi_serum_dex_permissioned::{Context, MarketMiddleware};
use solana_program::clock::Clock;

use crate::{errors, state::option_market::OptionMarket};

pub struct Validation {
    pub market_auth_bump: u8,
}

impl Validation {
    pub fn new() -> Self {
        Validation {
            market_auth_bump: 0,
        }
    }
}

impl MarketMiddleware for Validation {
    fn instruction(&mut self, data: &mut &[u8]) -> Result<()> {
        // Stripe the Validation diccriminator
        let disc = data[0];
        *data = &data[1..];
        // 6 is the Prune insruction, strip and set the marketAuhtorityBump
        if disc == 6 {
            self.market_auth_bump = data[0];
            *data = &data[1..];
        }
        Ok(())
    }

    fn prune(&self, ctx: &mut Context, _limit: &mut u16) -> Result<()> {
        // Validate that the OptionMarket has expired
        // deserialize the OptionMarket
        let option_market_account = ctx.accounts[0].clone();
        ctx.accounts = (&ctx.accounts[1..]).to_vec();
        let option_market_acct = Account::<OptionMarket>::try_from(&option_market_account)?;
        if option_market_acct.into_inner().expiration_unix_timestamp >= Clock::get()?.unix_timestamp
        {
            return Err(errors::ErrorCode::CannotPruneActiveMarket.into());
        }
        // Sign with the seeds
        ctx.accounts[3].is_signer = true;
        let seeds = vec![
            b"open-orders-init".to_vec(),
            ctx.dex_program_id.as_ref().to_vec(),
            // The serum market address
            ctx.accounts[0].key.as_ref().to_vec(),
            // this needs to be the market authority bump seed
            vec![self.market_auth_bump],
        ];
        ctx.seeds.push(seeds);
        Ok(())
    }

    fn fallback(&self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

pub mod referral {
    solana_program::declare_id!("6xUQFHLbbfhayBwLBNSMfZfCmHNJJujWsS88qCtGfWdn");
}
