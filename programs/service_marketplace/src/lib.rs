use anchor_lang::prelude::*;
use anchor_spl::token::{self, spl_token, Mint, MintTo, SetAuthority, Token, TokenAccount};
use spl_token::instruction::AuthorityType;

declare_id!("5HGnVbKouqv4Tyzmy2iEtCAvm1fswn6wdkDrdgdApMv1");

#[program]
pub mod service_marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn list_service(
        ctx: Context<ListService>,
        service_name: String,
        description: String,
        price: u64,
        is_soulbound: bool,
    ) -> Result<()> {
        let service_account = &mut ctx.accounts.service_account;
        service_account.vendor = *ctx.accounts.vendor.key;
        service_account.service_name = service_name;
        service_account.description = description;
        service_account.price = price;
        service_account.is_soulbound = is_soulbound;
        service_account.mint = *ctx.accounts.mint.to_account_info().key;

        // Mint the NFT to the vendor
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.vendor_token_account.to_account_info(),
            authority: ctx.accounts.vendor.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, 1)?;

        // If the NFT is soulbound, set the mint authority to zero address
        if is_soulbound {
            let cpi_accounts = SetAuthority {
                account_or_mint: ctx.accounts.mint.to_account_info(),
                current_authority: ctx.accounts.vendor.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
            token::set_authority(
                cpi_ctx,
                AuthorityType::MintTokens,
                None,
            )?;
        }

        Ok(())
    }

    pub fn purchase_service(ctx: Context<PurchaseService>) -> Result<()> {
        let service_account = &ctx.accounts.service_account;

        // Transfer funds from consumer to vendor
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.consumer_token_account.to_account_info(),
            to: ctx.accounts.vendor_token_account.to_account_info(),
            authority: ctx.accounts.consumer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, service_account.price)?;

        // Transfer the NFT from vendor to consumer
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.vendor_nft_account.to_account_info(),
            to: ctx.accounts.consumer_nft_account.to_account_info(),
            authority: ctx.accounts.vendor.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, 1)?;

        Ok(())
    }

    pub fn resale_service(ctx: Context<ResaleService>, sale_price: u64) -> Result<()> {
        let service_account = &ctx.accounts.service_account;
        let royalty_fee = sale_price * 5 / 100;  // Example: 5% royalty

        // Transfer royalty fee to the vendor
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.reseller_token_account.to_account_info(),
            to: ctx.accounts.vendor_token_account.to_account_info(),
            authority: ctx.accounts.reseller.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, royalty_fee)?;

        // Transfer the remaining amount to the reseller
        let remaining_amount = sale_price - royalty_fee;
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.reseller_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.reseller.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, remaining_amount)?;

        // Transfer the NFT from reseller to buyer
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.reseller_nft_account.to_account_info(),
            to: ctx.accounts.buyer_nft_account.to_account_info(),
            authority: ctx.accounts.reseller.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, 1)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct ListService<'info> {
    #[account(init, payer = vendor, space = 8 + 32 + 4 + 50 + 4 + 100 + 8 + 1 + 32)]
    pub service_account: Account<'info, ServiceAccount>,
    #[account(mut)]
    pub vendor: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub vendor_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct PurchaseService<'info> {
    #[account(mut)]
    pub service_account: Account<'info, ServiceAccount>,
    #[account(mut)]
    pub vendor: Signer<'info>,
    #[account(mut)]
    pub consumer: Signer<'info>,
    #[account(mut)]
    pub vendor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub consumer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vendor_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub consumer_nft_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ResaleService<'info> {
    #[account(mut)]
    pub service_account: Account<'info, ServiceAccount>,
    #[account(mut)]
    pub vendor: Signer<'info>,
    #[account(mut)]
    pub reseller: Signer<'info>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub vendor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reseller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reseller_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_nft_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct ServiceAccount {
    pub vendor: Pubkey,
    pub service_name: String,
    pub description: String,
    pub price: u64,
    pub is_soulbound: bool,
    pub mint: Pubkey,
}
