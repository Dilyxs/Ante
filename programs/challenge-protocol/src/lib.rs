use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::MintTo;
use anchor_spl::token_interface;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
const ANCHOR_DISCRIMINATOR: usize = 8;
use anchor_lang::solana_program::sysvar::clock::Clock;
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
    pub fn deposite_ante_tokens(ctx: Context<DepositeVault>, ante_token_count: u64) -> Result<()> {
        let user_ata = ctx.accounts.owner_ata.to_account_info();
        let vault_ata = ctx.accounts.vault_ata.to_account_info();
        let authority = ctx.accounts.owner.to_account_info();
        let mint = ctx.accounts.mint.to_account_info();
        let token_program = ctx.accounts.token_program.to_account_info();

        let transfer_accounts = anchor_spl::token_2022::TransferChecked {
            from: user_ata,
            to: vault_ata,
            authority,
            mint,
        };
        let seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.vault_authority]]];
        let cpi_transfer_ctx = CpiContext::new_with_signer(token_program, transfer_accounts, seeds);
        anchor_spl::token_2022::transfer_checked(
            cpi_transfer_ctx,
            ante_token_count,
            TOKEN_DEMICAL,
        )?;
        ctx.accounts.user_balance_info.balance = ante_token_count;
        Ok(())
    }
    pub fn withdraw_ante_tokens(
        ctx: Context<WithDrawFromVault>,
        ante_token_count: u64,
    ) -> Result<()> {
        if ctx.accounts.user_balance_info.balance < ante_token_count {
            return err!(ChallengeProtocolError::InsufficientAnteTokens);
        }
        let user_ata = ctx.accounts.owner_ata.to_account_info();
        let vault_ata = ctx.accounts.vault_ata.to_account_info();
        let authority = ctx.accounts.owner.to_account_info();
        let mint = ctx.accounts.mint.to_account_info();
        let token_program = ctx.accounts.token_program.to_account_info();

        let transfer_accounts = anchor_spl::token_2022::TransferChecked {
            from: vault_ata,
            to: user_ata,
            authority,
            mint,
        };
        let seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.vault_authority]]];
        let cpi_transfer_ctx = CpiContext::new_with_signer(token_program, transfer_accounts, seeds);
        anchor_spl::token_2022::transfer_checked(
            cpi_transfer_ctx,
            ante_token_count,
            TOKEN_DEMICAL,
        )?;
        ctx.accounts.user_balance_info.balance -= ante_token_count;
        Ok(())
    }
    pub fn upload_new_poster(
        ctx: Context<UploadNewPoster>,
        bounty_type: BountyType,
        bounty_topic: BountyTopic,
        deadline: u64,
        potential_answer: Option<[u8; 33]>,
        bounty_minimum_gain: u64,
    ) -> Result<()> {
        //checker whether user has enough ante tokens to post a poster
        ctx.accounts.data.bounty_id = ctx.accounts.vault_global_state.bounty_counter;
        ctx.accounts.vault_global_state.bounty_counter += 1;
        ctx.accounts.data.bounty_type = bounty_type.clone();
        ctx.accounts.data.bounty_minimum_gain = bounty_minimum_gain;
        ctx.accounts.data.bounty_topic = bounty_topic.clone();
        ctx.accounts.data.deadline = deadline;
        ctx.accounts.data.current_time = Clock::get()?.unix_timestamp as u64;
        ctx.accounts.data.potential_answer = potential_answer;

        emit!(PosterCreated {
            publisher: ctx.accounts.owner.key(),
            poster_info: Poster {
                bounty_id: ctx.accounts.data.bounty_id,
                bounty_type,
                bounty_topic,
                bounty_minimum_gain,
                deadline,
                current_time: ctx.accounts.data.current_time,
                potential_answer,
            }
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct WithDrawFromVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=owner,
    )]
    pub owner_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,
        seeds=[b"Ante"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
        seeds=[b"authority"],
        bump)]
    ///CHECK: vault_authority is safe
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=vault_authority,
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,
    seeds=[b"user_balance_info", owner.key().as_ref()],
    bump)]
    pub user_balance_info: Account<'info, UserBalance>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
#[derive(Accounts)]
pub struct DepositeVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=owner,
    )]
    pub owner_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,
        seeds=[b"Ante"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
        seeds=[b"authority"],
        bump)]
    ///CHECK: vault_authority is safe
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=vault_authority,
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(init_if_needed,
    space=ANCHOR_DISCRIMINATOR+ UserBalance::INIT_SPACE,
    payer=owner,seeds=[b"user_balance_info", owner.key().as_ref()],
    bump)]
    pub user_balance_info: Account<'info, UserBalance>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
#[derive(Accounts)]
pub struct UploadNewPoster<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut,
    seeds=[b"Ante"],
    bump)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
    associated_token::mint = mint,
    associated_token::authority = owner)]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,
    seeds=[b"vault_global_state"],
    bump)]
    pub vault_global_state: Account<'info, VaultGlobalState>,
    #[account(init,
        space=Poster::INIT_SPACE + ANCHOR_DISCRIMINATOR,
    payer=owner,
    seeds=[b"poster",vault_global_state.bounty_counter.to_le_bytes().as_ref(), owner.key().as_ref()],
    bump)]
    pub data: Account<'info, Poster>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
    #[account(init,
        space=ANCHOR_DISCRIMINATOR+ VaultGlobalState::INIT_SPACE,
        payer=signer,
        seeds=[b"vault_global_state"],
        bump
    )]
    pub vault_global_state: Account<'info, VaultGlobalState>,
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
    bounty_id: u64,
    bounty_type: BountyType,
    bounty_topic: BountyTopic,
    bounty_minimum_gain: u64,
    deadline: u64,
    current_time: u64,
    potential_answer: Option<[u8; 33]>,
}

#[account]
pub struct PosterResponse {
    time: u64,
    poster_id: u64,
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
    #[msg("poster deadline has passed")]
    InsufficientAnteTokens,
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
}
#[derive(InitSpace)]
#[account]
pub struct UserBalance {
    pub user: Pubkey,
    pub balance: u64,
}
#[derive(InitSpace)]
#[account]
pub struct VaultGlobalState {
    pub bounty_counter: u64,
}
#[event]
pub struct PosterCreated {
    pub publisher: Pubkey,
    pub poster_info: Poster,
}
