use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::clock::Clock;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::MintTo;
use anchor_spl::token_2022::TransferChecked;
use anchor_spl::token_interface;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
pub mod helper;
const TOKEN_DEMICAL: u8 = 6;
const ANCHOR_DISCRIMINATOR: usize = 8;
const ONE_WEEK_IN_SECONDS: u64 = 604800;
declare_id!("7sGGy4oHLkKsfwzbb8fw2z8ZXH7ZMFhUeD62Z1y379tS");
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

        ctx.accounts.data.publisher = ctx.accounts.owner.key();
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
            current_time <= ctx.accounts.poster_info.deadline,
            ChallengeProtocolError::PosterDeadlinePassed
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
        ctx.accounts.poster_response.answer_id = ctx.accounts.vault_global_state.response_counter;
        ctx.accounts.poster_response.time = Clock::get()?.unix_timestamp as u64;
        ctx.accounts.poster_response.answerer = ctx.accounts.answerer.key();
        ctx.accounts.vault_global_state.response_counter += 1;
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
        // ensure the poster deadline has passed
        require!(
            Clock::get()?.unix_timestamp as u64 > ctx.accounts.poster_info.deadline,
            ChallengeProtocolError::PosterDeadlineNotPassed
        );
        require!(
            u64::checked_add(
                ctx.accounts.user_balance_info.balance,
                ctx.accounts.poster_info.bounty_minimum_gain
            )
            .is_some(),
            ChallengeProtocolError::OverflowError
        );
        let token_prgram = ctx.accounts.token_program.to_account_info();
        let req = TransferChecked {
            from: ctx.accounts.vault_ata.to_account_info(),
            to: ctx.accounts.poster_publisher_ata.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };
        let seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.vault_authority]]];
        let cpi_transfer_ctx = CpiContext::new_with_signer(token_prgram, req, seeds);
        token_interface::transfer_checked(
            cpi_transfer_ctx,
            ctx.accounts.poster_info.bounty_minimum_gain,
            TOKEN_DEMICAL,
        )?;
        ctx.accounts.user_balance_info.balance += ctx.accounts.poster_info.bounty_minimum_gain;
        Ok(())
    }
    pub fn post_poster_solution(
        ctx: Context<PostPosterSolution>,
        poster_id: u64,
        answer: String,
        hash: String,
    ) -> Result<()> {
        require!(
            Clock::get()?.unix_timestamp as u64 > ctx.accounts.poster_info.deadline,
            ChallengeProtocolError::PosterDeadlineNotPassed
        );
        ctx.accounts.publisher_decrypted_answer.poster_id = poster_id;
        ctx.accounts.publisher_decrypted_answer.answer = answer.clone();
        ctx.accounts.publisher_decrypted_answer.hash = hash.clone();
        emit!(PosterPublishAnswered {
            publisher: ctx.accounts.publisher.key(),
            poster_publisher_decrypted_answer: PosterPublisherDecryptedAnswerInfo {
                poster_id,
                answer,
                hash
            }
        });
        Ok(())
    }
    pub fn post_answerer_decrypted_answer(
        ctx: Context<PostAnswererDecryptedAnswer>,
        poster_id: u64,
        answer: String,
        hash: String,
    ) -> Result<()> {
        require!(
            Clock::get()?.unix_timestamp as u64 > ctx.accounts.poster_info.deadline,
            ChallengeProtocolError::PosterDeadlineNotPassed
        );
        ctx.accounts.answerer_decrypted_answer.poster_id = poster_id;
        ctx.accounts.answerer_decrypted_answer.answer = answer.clone();
        ctx.accounts.answerer_decrypted_answer.hash = hash.clone();
        emit!(AnswererDecryptedAnswerPosted {
            answerer: ctx.accounts.answerer.key(),
            answerer_decrypted_answer: PosterAnswererDecryptedAnswerInfo {
                poster_id,
                answer,
                hash
            }
        });
        Ok(())
    }
    pub fn refund_answerer_where_poster_didnt_post_solution(
        ctx: Context<RefundAnswererWherePosterDidntPostSolution>,
        poster_id: u64,
    ) -> Result<()> {
        require!(
            Clock::get()?.unix_timestamp as u64
                > ctx.accounts.poster_info.deadline + ONE_WEEK_IN_SECONDS,
            ChallengeProtocolError::OneWeekDeadlineNotPassed
        );
        require!(
            u64::checked_add(
                ctx.accounts.user_balance_info.balance,
                ctx.accounts.poster_info.submission_cost
            )
            .is_some(),
            ChallengeProtocolError::OverflowError
        );
        let token_prgram = ctx.accounts.token_program.to_account_info();
        let req = TransferChecked {
            from: ctx.accounts.vault_ata.to_account_info(),
            to: ctx.accounts.poster_answerer_ata.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };
        let seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.vault_authority]]];
        let cpi_transfer_ctx = CpiContext::new_with_signer(token_prgram, req, seeds);
        token_interface::transfer_checked(
            cpi_transfer_ctx,
            ctx.accounts.poster_info.submission_cost,
            TOKEN_DEMICAL,
        )?;
        ctx.accounts.user_balance_info.balance += ctx.accounts.poster_info.submission_cost;
        emit!(PublisherNotResponded {
            poster_id,
            published_id: ctx.accounts.poster_info.publisher,
        });
        Ok(())
    }
    pub fn post_poster_winner(
        ctx: Context<PostPosterWinner>,
        poster_id: u64,
        winner: Pubkey,
        contestains_count: u64,
    ) -> Result<()> {
        require!(
            Clock::get()?.unix_timestamp as u64
                > ctx.accounts.poster_info.deadline + ONE_WEEK_IN_SECONDS,
            ChallengeProtocolError::OneWeekDeadlineNotPassed
        );
        let token_prgram = ctx.accounts.token_program.to_account_info();
        let req = TransferChecked {
            from: ctx.accounts.vault_ata.to_account_info(),
            to: ctx.accounts.winner_ata.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };
        let seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.vault_authority]]];
        let cpi_transfer_ctx = CpiContext::new_with_signer(token_prgram, req, seeds);
        // compute winner share using checked arithmetic to avoid overflow
        let total_from_contestants = ctx
            .accounts
            .poster_info
            .submission_cost
            .checked_mul(contestains_count)
            .ok_or(ChallengeProtocolError::OverflowError)?;
        let winner_share = total_from_contestants
            .checked_add(ctx.accounts.poster_info.bounty_minimum_gain)
            .ok_or(ChallengeProtocolError::OverflowError)?;

        token_interface::transfer_checked(cpi_transfer_ctx, winner_share, TOKEN_DEMICAL)?;
        // Keep winner's internal balance tracking in sync with transfer.
        ctx.accounts.user_balance_info.balance = ctx
            .accounts
            .user_balance_info
            .balance
            .checked_add(winner_share)
            .ok_or(ChallengeProtocolError::OverflowError)?;
        ctx.accounts.post_winner.poster_id = poster_id;
        ctx.accounts.post_winner.winner_id = winner;
        emit!(PosterWinnerPostedEvent { poster_id, winner });
        Ok(())
    }
    pub fn vote_for_winner(
        ctx: Context<VotingWinner>,
        poster_id: u64,
        winner: Pubkey,
    ) -> Result<()> {
        require!(
            winner != ctx.accounts.voter.key.clone(),
            ChallengeProtocolError::UserVotedForThemselves
        );
        ctx.accounts.vote_for_winner.poster_id = poster_id;
        ctx.accounts.vote_for_winner.voter = ctx.accounts.voter.key();
        ctx.accounts.vote_for_winner.winner_vote = winner;
        ctx.accounts.vote_for_winner.response_id = ctx.accounts.poster_response.answer_id;
        emit!(VoteForWinnerPosted {
            poster_id,
            voter: ctx.accounts.voter.key(),
            winner
        });
        Ok(())
    }
}
#[derive(Accounts)]
#[instruction(poster_id: u64)]
pub struct VotingWinner<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(mut,
    seeds=[b"poster", poster_id.to_le_bytes().as_ref()],
    bump)]
    pub poster_info: Account<'info, Poster>,
    #[account(mut,
    seeds=[b"poster_answerer_decrypted_answer", poster_id.to_le_bytes().as_ref(), voter.key().as_ref()],
    bump)]
    pub poster_decrypted_response: Account<'info, PosterAnswererDecryptedAnswer>,
    #[account(init,
        payer=voter,
        space=ANCHOR_DISCRIMINATOR + VoteForWinner::INIT_SPACE,
    seeds=[b"vote_for_winner", poster_id.to_le_bytes().as_ref(), voter.key().as_ref()],
    bump)]
    pub vote_for_winner: Account<'info, VoteForWinner>,
    #[account(mut,
    seeds=[b"poster_response", poster_id.to_le_bytes().as_ref(), voter.key().as_ref()],
    bump)]
    pub poster_response: Account<'info, PosterResponse>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poster_id: u64)]
pub struct PostPosterWinner<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut,
    seeds=[b"config"],
    bump,
    has_one = admin
    )]
    pub config: Account<'info, Config>,
    #[account(mut,
    seeds=[b"post_winner", poster_id.to_le_bytes().as_ref()],
    bump)]
    pub post_winner: Account<'info, PostingWinner>,
    #[account(mut,
    seeds=[b"poster", poster_id.to_le_bytes().as_ref()],
    bump)]
    pub poster_info: Account<'info, Poster>,
    #[account(mut,
    seeds=[b"user_balance_info", winner.key().as_ref()],
        bump)]
    pub user_balance_info: Account<'info, UserBalance>,
    #[account(mut,
seeds=[b"authority"],
bump)]
    ///CHECK: vault_authority is safe
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut,
associated_token::mint=mint,
associated_token::authority=vault_authority,)]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    ///CHECK: winner is safe since we are only transfering tokens to them
    pub winner: UncheckedAccount<'info>,
    #[account(mut,
associated_token::mint=mint,
associated_token::authority=winner,)]
    pub winner_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,
    seeds=[b"Ante"],
    bump
)]
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
#[derive(Accounts)]
#[instruction(poster_id: u64)]
pub struct RefundAnswererWherePosterDidntPostSolution<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut,
    seeds=[b"config"],
    bump,
    has_one = admin
    )]
    pub config: Account<'info, Config>,
    #[account(mut,
    seeds=[b"authority"],
    bump)]
    ///CHECK: vault_authority is safe
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut,
        seeds=[b"Ante"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
    associated_token::mint = mint,
        associated_token::token_program = token_program,
    associated_token::authority=vault_authority,
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,
    ///CHECK: poster_answerer is safe since we are only reading their pubkey for PDA checks
    pub poster_answerer: UncheckedAccount<'info>,
    #[account(mut,
    seeds=[b"poster_response", poster_id.to_le_bytes().as_ref(), poster_answerer.key().as_ref()],
    bump)]
    pub poster_response: Account<'info, PosterResponse>,
    #[account(mut,
    seeds=[b"user_balance_info", poster_answerer.key().as_ref()],
        bump)]
    pub user_balance_info: Account<'info, UserBalance>,
    #[account(mut,
    seeds=[b"poster", poster_id.to_le_bytes().as_ref()],
    bump)]
    pub poster_info: Account<'info, Poster>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=poster_answerer,
    )]
    pub poster_answerer_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(poster_id: u64)]
pub struct PostAnswererDecryptedAnswer<'info> {
    #[account(mut)]
    pub answerer: Signer<'info>,
    #[account(mut,
        seeds=[b"poster_response", poster_id.to_le_bytes().as_ref(), answerer.key().as_ref()],
        bump,
        has_one = answerer
    )]
    pub poster_response: Account<'info, PosterResponse>,
    #[account(init,
        space=ANCHOR_DISCRIMINATOR + PosterAnswererDecryptedAnswer::INIT_SPACE,
        payer=answerer,
        seeds=[b"poster_answerer_decrypted_answer", poster_id.to_le_bytes().as_ref(), answerer.key().as_ref()],
        bump)]
    pub answerer_decrypted_answer: Account<'info, PosterAnswererDecryptedAnswer>,
    #[account(mut,
        seeds=[b"poster", poster_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poster_info: Account<'info, Poster>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
#[instruction(poster_id: u64)]
pub struct PostPosterSolution<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    #[account(mut,
        seeds=[b"poster", poster_id.to_le_bytes().as_ref()],
        bump,
        has_one = publisher
    )]
    pub poster_info: Account<'info, Poster>,
    #[account(init,
        space=ANCHOR_DISCRIMINATOR + PosterPublisherDecryptedAnswer::INIT_SPACE,
        payer=publisher,
        seeds=[b"poster_decrypted_answer", poster_id.to_le_bytes().as_ref(), publisher.key().as_ref()],
        bump)]
    pub publisher_decrypted_answer: Account<'info, PosterPublisherDecryptedAnswer>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
#[instruction(poster_id: u64)]
pub struct NoSubmissionPoster<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut,
    seeds=[b"config"],
    bump,
    has_one = admin
    )]
    pub config: Account<'info, Config>,
    #[account(mut,
    seeds=[b"authority"],
    bump)]
    ///CHECK: vault_authority is safe
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut,
        seeds=[b"Ante"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=vault_authority,
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,
    seeds=[b"poster", poster_id.to_le_bytes().as_ref()],
    bump)]
    pub poster_info: Account<'info, Poster>,
    ///CHECK: poster_publisher is safe since we are only reading data from them
    pub poster_publisher: UncheckedAccount<'info>,
    #[account(mut,
    seeds=[b"user_balance_info", poster_publisher.key().as_ref()],
    bump)]
    pub user_balance_info: Account<'info, UserBalance>,
    #[account(mut,
    associated_token::mint=mint,
    associated_token::authority=poster_publisher)]
    pub poster_publisher_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
        seeds=[b"poster", poster_id.to_le_bytes().as_ref()],
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

    #[account(mut,
    seeds=[b"vault_global_state"],
    bump)]
    pub vault_global_state: Account<'info, VaultGlobalState>,
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
    seeds=[b"poster",vault_global_state.bounty_counter.to_le_bytes().as_ref()],
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
    #[account(init,
    seeds=[b"post_winner", vault_global_state.bounty_counter.to_le_bytes().as_ref()],
    bump,
    space=ANCHOR_DISCRIMINATOR + PostingWinner::INIT_SPACE,
    payer=owner)]
    pub posting_winner: Account<'info, PostingWinner>,
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

#[derive(Clone, InitSpace, AnchorDeserialize, AnchorSerialize, Debug)]
pub enum BountyType {
    OpenEnded,
    DirectAnswer,
}
#[derive(Clone, InitSpace, AnchorDeserialize, AnchorSerialize, Debug)]
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
    pub publisher: Pubkey,
    pub bounty_id: u64,
    pub bounty_type: BountyType,
    pub bounty_topic: BountyTopic,
    pub bounty_minimum_gain: u64,
    pub submission_cost: u64,
    pub deadline: u64,
    pub current_time: u64,
    pub potential_answer: Option<[u8; 33]>,
}
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct PosterInfo {
    pub bounty_id: u64,
    pub bounty_type: BountyType,
    pub bounty_topic: BountyTopic,
    pub bounty_minimum_gain: u64,
    pub submission_cost: u64,
    pub deadline: u64,
    pub current_time: u64,
    pub potential_answer: Option<[u8; 33]>,
}

#[account]
#[derive(InitSpace)]
pub struct PosterResponse {
    pub answerer: Pubkey,
    pub time: u64,
    pub poster_id: u64,
    pub answer_id: u64,
    pub answer: Option<[u8; 33]>,
}
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct PosterResponseInfo {
    pub time: u64,
    pub poster_id: u64,
    pub answer: Option<[u8; 33]>,
}

#[account]
pub struct PosterWinner {
    pub poster_id: u64,
    pub winner_id: Pubkey,
}
#[account]
#[derive(InitSpace)]
pub struct PosterPublisherDecryptedAnswer {
    pub poster_id: u64,
    #[max_len(64)]
    pub answer: String,
    #[max_len(32)]
    pub hash: String,
}
#[account]
#[derive(InitSpace)]
pub struct PosterAnswererDecryptedAnswer {
    pub poster_id: u64,
    #[max_len(64)]
    pub answer: String,
    #[max_len(32)]
    pub hash: String,
}
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct PosterPublisherDecryptedAnswerInfo {
    pub poster_id: u64,
    pub answer: String,
    pub hash: String,
}
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct PosterAnswererDecryptedAnswerInfo {
    pub poster_id: u64,
    pub answer: String,
    pub hash: String,
}
#[account]
#[derive(InitSpace)]
pub struct PostingWinner {
    pub poster_id: u64,
    pub winner_id: Pubkey,
}
#[account]
#[derive(InitSpace)]
pub struct VoteForWinner {
    pub poster_id: u64,
    pub response_id: u64,
    pub voter: Pubkey,
    pub winner_vote: Pubkey,
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
    #[msg("overflow error")]
    OverflowError,
    #[msg("1 week deadline has not passed yet")]
    OneWeekDeadlineNotPassed,
    #[msg("user cannot vote for themselves")]
    UserVotedForThemselves,
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
    pub response_counter: u64,
}
#[derive(Debug, Clone)]
#[event]
pub struct PosterCreated {
    pub publisher: Pubkey,
    pub poster_info: PosterInfo,
}
#[derive(Debug, Clone)]
#[event]
pub struct PosterAnswered {
    pub answerer: Pubkey,
    pub poster_response: PosterResponseInfo,
}
#[derive(Debug, Clone)]
#[event]
pub struct PosterPublishAnswered {
    pub publisher: Pubkey,
    pub poster_publisher_decrypted_answer: PosterPublisherDecryptedAnswerInfo,
}
#[derive(Debug, Clone)]
#[event]
pub struct AnswererDecryptedAnswerPosted {
    pub answerer: Pubkey,
    pub answerer_decrypted_answer: PosterAnswererDecryptedAnswerInfo,
}
#[event]
#[derive(Debug, Clone)]
pub struct PosterWinnerPostedEvent {
    pub poster_id: u64,
    pub winner: Pubkey,
}
#[derive(Debug, Clone)]
#[event]
pub struct PublisherNotResponded {
    pub poster_id: u64,
    pub published_id: Pubkey,
}
#[derive(Debug, Clone)]
#[event]
pub struct VoteForWinnerPosted {
    pub poster_id: u64,
    pub voter: Pubkey,
    pub winner: Pubkey,
}
