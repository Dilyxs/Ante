use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::MintTo;
use anchor_spl::token_interface;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
const ANCHOR_DISCRIMINATOR: usize = 8;
const TOKEN_DEMICAL: u8 = 6;
pub mod helper;
declare_id!("FraezYKQ1zKbysJJK7jYGr8J2ZVt5Cyz3GrQ3WSDLQyS");

#[program]
pub mod challenge_protocol {
    use super::*;
    pub fn initialize(ctx: Context<InitializeProgram>) -> Result<()> {
        ctx.accounts.config.admin = ctx.accounts.signer.key();
        Ok(())
    }
    pub fn requestAnteTokens(
        ctx: Context<GetAndeTokensFaucet>,
        ante_token_count: u64,
    ) -> Result<()> {
        if ante_token_count != 1
            && ante_token_count != 2
            && ante_token_count != 4
            && ante_token_count != 8
        {
            return err!(ChallengeProtocolError::IncorrectTokenRequestAmount);
        }
        let asker_ata = ctx.accounts.asker_ata.to_account_info();
        let token_program = ctx.accounts.token_program.to_account_info();
        let vault_authority = ctx.accounts.vault_authority.to_account_info();
        let mint = ctx.accounts.mint.to_account_info();

        let seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.vault_authority]]];
        let req = MintTo {
            mint,
            to: asker_ata,
            authority: vault_authority,
        };
        let cpi_req = CpiContext::new_with_signer(token_program, req, seeds);
        token_interface::mint_to(cpi_req, ante_token_count)?;

        Ok(())
    }
}
#[derive(Accounts)]
pub struct GetAndeTokensFaucet<'info> {
    pub admin: Signer<'info>,
    #[account(mut,
        seeds=[b"Ante"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    ///CHECK: get asker, they are safe since we are only transfering tokens to them
    pub asker: UncheckedAccount<'info>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=asker,
    )]
    pub asker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
    seeds=[b"authority"],
    bump
    )]
    ///CHECK: vault_authority is safe
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=vault_authority,
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,
    seeds=[b"config"],
    bump,
    has_one = admin
    )]
    pub config: Account<'info, Config>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
    seeds=[b"authority"],
    bump)]
    ///CHECK: vault_authority is safe
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        seeds=[b"Ante"],
        bump,
        mint::decimals = TOKEN_DEMICAL,
        mint::authority = vault_authority.key(),
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(init,
        payer=signer,
        associated_token::mint=mint,
        associated_token::token_program=token_program,
        associated_token::authority=vault_authority,
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(init,
        space=ANCHOR_DISCRIMINATOR+ Config::INIT_SPACE,
        payer=signer,
        seeds=[b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Clone, InitSpace, AnchorDeserialize, AnchorSerialize)]
pub enum BountyType {
    OpenEnded,
    DirectAnswer,
}
#[derive(Clone, InitSpace, AnchorDeserialize, AnchorSerialize)]
pub enum BountyTopic {
    NumberTheory,
    CryptoPuzzle,
    ReverseEng,
    NumericalTrivial,
    PrivateKeyPuzzle,
}
#[derive(InitSpace)]
#[account]
pub struct Poster {
    id: [u8; 32],
    bounty_type: BountyType,
    bounty_topic: BountyTopic,
    deadline: u64,
    current_time: u64,
    potential_answer: Option<[u8; 33]>,
}

#[account]
pub struct PosterResponse {
    time: u64,
    poster_id: [u8; 32],
    answer: Option<[u8; 33]>,
}

#[account]
pub struct PosterWinner {
    pub poster_id: Pubkey,
    pub winner_id: Pubkey,
}

#[error_code]
pub enum ChallengeProtocolError {
    #[msg("incorrect token request amount")]
    IncorrectTokenRequestAmount,
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
}
