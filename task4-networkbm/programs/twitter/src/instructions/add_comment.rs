//-------------------------------------------------------------------------------
///
/// TASK: Implement the add comment functionality for the Twitter program
/// 
/// Requirements:
/// - Validate that comment content doesn't exceed maximum length
/// - Initialize a new comment account with proper PDA seeds
/// - Set comment fields: content, author, parent tweet, and bump
/// - Use content hash in PDA seeds for unique comment identification
/// 
///-------------------------------------------------------------------------------

use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;
use anchor_lang::Discriminator;
use crate::states::{Tweet, Comment, COMMENT_SEED};
use crate::errors::TwitterError;

#[derive(Accounts)]
pub struct AddCommentContext<'info> {
    #[account(mut)]
    pub comment_author: Signer<'info>,
    
    /// CHECK: validate
    #[account(mut)]
    pub comment: UncheckedAccount<'info>,

    #[account(mut)]
    pub tweet: Account<'info, Tweet>,

    pub system_program: Program<'info, System>,
}

pub fn add_comment(ctx: Context<AddCommentContext>, comment_content: String) -> Result<()> {
    require!(
        comment_content.as_bytes().len() <= 500,
        TwitterError::CommentTooLong
    );

    let content_hash = hash(comment_content.as_bytes());
    let author_key = ctx.accounts.comment_author.key();
    let tweet_key = ctx.accounts.tweet.key();
    
    let seeds = &[
        COMMENT_SEED.as_bytes(),
        author_key.as_ref(),
        content_hash.as_ref(),
        tweet_key.as_ref(),
    ];
    let (expected_pda, bump) = Pubkey::find_program_address(seeds, ctx.program_id);
    
    require_keys_eq!(
        ctx.accounts.comment.key(),
        expected_pda,
        TwitterError::ContentTooLong
    );

    let space = 8 + 32 + 32 + (4 + 500) + 1;
    let lamports = Rent::get()?.minimum_balance(space);

    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            ctx.accounts.comment_author.key,
            ctx.accounts.comment.key,
            lamports,
            space as u64,
            ctx.program_id,
        ),
        &[
            ctx.accounts.comment_author.to_account_info(),
            ctx.accounts.comment.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[
            COMMENT_SEED.as_bytes(),
            author_key.as_ref(),
            content_hash.as_ref(),
            tweet_key.as_ref(),
            &[bump],
        ]],
    )?;

    let comment_account = &ctx.accounts.comment;
    let mut data = comment_account.try_borrow_mut_data()?;
    
    let discriminator = Comment::DISCRIMINATOR;
    data[0..8].copy_from_slice(&discriminator);

    let mut cursor = &mut data[8..];
    author_key.serialize(&mut cursor)?;
    tweet_key.serialize(&mut cursor)?;
    comment_content.serialize(&mut cursor)?;
    bump.serialize(&mut cursor)?;

    Ok(())
}