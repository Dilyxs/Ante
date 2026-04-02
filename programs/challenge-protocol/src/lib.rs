use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::MintTo;
use anchor_spl::token_interface;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
const ANCHOR_DISCRIMINATOR: usize = 8;
use anchor_lang::solana_program::sysvar::clock::Clock;
use anchor_spl::token_2022::TransferChecked;
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

        let transfer_accounts = TransferChecked {
            from: user_ata,
            to: vault_ata,
            authority,
            mint,
        };
        let cpi_transfer_ctx = CpiContext::new(token_program, transfer_accounts);
        token_interface::transfer_checked(cpi_transfer_ctx, ante_token_count, TOKEN_DEMICAL)?;
        ctx.accounts.user_balance_info.balance += ante_token_count;
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
        let authority = ctx.accounts.vault_authority.to_account_info();
        let mint = ctx.accounts.mint.to_account_info();
        let token_program = ctx.accounts.token_program.to_account_info();

        let transfer_accounts = TransferChecked {
            from: vault_ata,
            to: user_ata,
            authority,
            mint,
        };
        let seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.vault_authority]]];
        let cpi_transfer_ctx = CpiContext::new_with_signer(token_program, transfer_accounts, seeds);
        token_interface::transfer_checked(cpi_transfer_ctx, ante_token_count, TOKEN_DEMICAL)?;
        ctx.accounts.user_balance_info.balance -= ante_token_count;
        Ok(())
    }
    pub fn upload_new_poster(
        ctx: Context<UploadNewPoster>,
        bounty_minimum_gain: u64,
        bounty_type: BountyType,
        bounty_topic: BountyTopic,
        deadline: u64,
        potential_answer: Option<[u8; 33]>,
        submission_cost: u64,
    ) -> Result<()> {
        require!(
            u64::checked_sub(ctx.accounts.user_balance_info.balance, bounty_minimum_gain).is_some(),
            ChallengeProtocolError::InsufficientAnteTokens
        );
        let token_prgram = ctx.accounts.token_program.to_account_info();
        let req = TransferChecked {
            from: ctx.accounts.user_ata.to_account_info(),
            to: ctx.accounts.vault_ata.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };
        let cpi_transfer_ctx = CpiContext::new(token_prgram, req);
        token_interface::transfer_checked(cpi_transfer_ctx, bounty_minimum_gain, TOKEN_DEMICAL)?;

        ctx.accounts.data.bounty_id = ctx.accounts.vault_global_state.bounty_counter;
        ctx.accounts.vault_global_state.bounty_counter += 1;
        ctx.accounts.data.submission_cost = submission_cost;
        ctx.accounts.data.bounty_type = bounty_type.clone();
        ctx.accounts.data.bounty_minimum_gain = bounty_minimum_gain;
        ctx.accounts.data.bounty_topic = bounty_topic.clone();
        ctx.accounts.data.deadline = deadline;
        ctx.accounts.data.current_time = Clock::get()?.unix_timestamp as u64;
        ctx.accounts.data.potential_answer = potential_answer;
        ctx.accounts.user_balance_info.balance -= bounty_minimum_gain;
        emit!(PosterCreated {
            publisher: ctx.accounts.owner.key(),
            poster_info: PosterInfo {
                bounty_id: ctx.accounts.data.bounty_id,
                submission_cost,
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
    pub fn answer_poster(
        ctx: Context<AnswerPoster>,
        poster_id: u64,
        answer: [u8; 33],
    ) -> Result<()> {
        require!(
            u64::checked_sub(
                ctx.accounts.user_balance_info.balance,
                ctx.accounts.poster_info.submission_cost
            )
            .is_some(),
            ChallengeProtocolError::InsufficientAnteTokens
        );
        let current_time = Clock::get()?.unix_timestamp as u64;
        require!(
            ctx.accounts.poster_info.deadline <= current_time,
            ChallengeProtocolError::PosterDeadlineNotPassed
        );
        let token_prgram = ctx.accounts.token_program.to_account_info();
        let req = TransferChecked {
            from: ctx.accounts.answerer_ata.to_account_info(),
            to: ctx.accounts.vault_ata.to_account_info(),
            authority: ctx.accounts.answerer.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };
        let cpi_transfer_ctx = CpiContext::new(token_prgram, req);
        token_interface::transfer_checked(
            cpi_transfer_ctx,
            ctx.accounts.poster_info.submission_cost,
            TOKEN_DEMICAL,
        )?;
        ctx.accounts.poster_response.time = Clock::get()?.unix_timestamp as u64;
        ctx.accounts.poster_response.poster_id = ctx.accounts.poster_info.bounty_id;
        ctx.accounts.poster_response.answer = Some(answer);
        ctx.accounts.user_balance_info.balance -= ctx.accounts.poster_info.submission_cost;

        emit!(PosterAnswered {
            answerer: ctx.accounts.answerer.key(),
            poster_response: PosterResponseInfo {
                time: ctx.accounts.poster_response.time,
                poster_id: ctx.accounts.poster_response.poster_id,
                answer: ctx.accounts.poster_response.answer
            }
        });

        Ok(())
    }
    pub fn no_submission_poster(ctx: Context<NoSubmissionPoster>, poster_id: u64) -> Result<()> {
        let poster_info = &ctx.accounts.poster_info;
        require!(
            Clock::get()?.unix_timestamp as u64 > poster_info.deadline,
            ChallengeProtocolError::InsufficientAnteTokens
        );
        Ok(())
    }
}

//TODO: add a constrant with instruction with poster_minimum_submission so user doens't waste their sol
#[derive(Accounts)]
#[instruction(poster_id: u64)]
pub struct AnswerPoster<'info> {
    #[account(mut)]
    pub answerer: Signer<'info>,
    #[account(mut,
        seeds=[b"Ante"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=answerer,
    )]
    pub answerer_ata: InterfaceAccount<'info, TokenAccount>,
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
        seeds=[b"user_balance_info", answerer.key().as_ref()],
        bump,
        constraint = user_balance_info.balance >= poster_info.submission_cost @ ChallengeProtocolError::InsufficientAnteTokens
    )]
    pub user_balance_info: Account<'info, UserBalance>,
    #[account(mut,
        seeds=[b"poster", poster_id.to_le_bytes().as_ref(), poster_publisher.key().as_ref()],
        bump,
    )]
    pub poster_info: Account<'info, Poster>,
    ///CHECK: poster_publisher is safe since we are only reading data from them
    pub poster_publisher: UncheckedAccount<'info>,
    #[account(init,
        space=ANCHOR_DISCRIMINATOR + PosterResponse::INIT_SPACE,
        payer=answerer,
        seeds=[b"poster_response", poster_id.to_le_bytes().as_ref(), answerer.key().as_ref()],
        bump)]
    pub poster_response: Account<'info, PosterResponse>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
#[instruction(bounty_minimum_gain: u64)]
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
    #[account(mut,
        seeds=[b"user_balance_info", owner.key().as_ref()],
        bump)]
    pub user_balance_info: Account<'info, UserBalance>,
    #[account(init,
        space=Poster::INIT_SPACE + ANCHOR_DISCRIMINATOR,
    payer=owner,
    seeds=[b"poster",vault_global_state.bounty_counter.to_le_bytes().as_ref(), owner.key().as_ref()],
    constraint = user_balance_info.balance >= bounty_minimum_gain,
    bump)]
    pub data: Account<'info, Poster>,
    #[account(mut,
    seeds=[b"authority"],
    bump)]
    ///CHECK: vault_authority is safe
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=vault_authority,)]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,
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
    submission_cost: u64,
    deadline: u64,
    current_time: u64,
    potential_answer: Option<[u8; 33]>,
}
#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct PosterInfo {
    bounty_id: u64,
    bounty_type: BountyType,
    bounty_topic: BountyTopic,
    bounty_minimum_gain: u64,
    submission_cost: u64,
    deadline: u64,
    current_time: u64,
    potential_answer: Option<[u8; 33]>,
}

#[account]
#[derive(InitSpace)]
pub struct PosterResponse {
    time: u64,
    poster_id: u64,
    answer: Option<[u8; 33]>,
}
#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct PosterResponseInfo {
    time: u64,
    poster_id: u64,
    answer: Option<[u8; 33]>,
}

#[account]
pub struct PosterWinner {
    pub poster_id: u64,
    pub winner_id: Pubkey,
}

#[error_code]
pub enum ChallengeProtocolError {
    #[msg("incorrect token request amount")]
    IncorrectTokenRequestAmount,
    #[msg("insufficient ante tokens in vault")]
    InsufficientAnteTokens,
    #[msg("poster deadline has not passed yet")]
    PosterDeadlineNotPassed,
    #[msg("poster deadline has passed, no submission allowed")]
    PosterDeadlinePassed,
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
    pub poster_info: PosterInfo,
}
#[event]
pub struct PosterAnswered {
    pub answerer: Pubkey,
    pub poster_response: PosterResponseInfo,
}
