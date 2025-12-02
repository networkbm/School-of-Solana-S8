//-------------------------------------------------------------------------------
///
/// TASK: Implement the add reaction functionality for the Twitter program
/// 
/// Requirements:
/// - Initialize a new reaction account with proper PDA seeds
/// - Increment the appropriate counter (likes or dislikes) on the tweet
/// - Set reaction fields: type, author, parent tweet, and bump
/// - Handle both Like and Dislike reaction types
/// 
///-------------------------------------------------------------------------------

use anchor_lang::prelude::*;
use crate::states::*;

#[derive(Accounts)]
pub struct AddReactionContext<'info> {
    #[account(mut)]
    pub reaction_author: Signer<'info>,

    #[account(
        init,
        payer = reaction_author,
        space = 8 + 32 + 32 + 1 + 1,
        seeds = [
            TWEET_REACTION_SEED.as_bytes(),
            reaction_author.key().as_ref(),
            tweet.key().as_ref(),
        ],
        bump
    )]
    pub tweet_reaction: Account<'info, Reaction>,

    #[account(mut)]
    pub tweet: Account<'info, Tweet>,

    pub system_program: Program<'info, System>,
}

pub fn add_reaction(ctx: Context<AddReactionContext>, reaction: ReactionType) -> Result<()> {
    let tweet_reaction = &mut ctx.accounts.tweet_reaction;
    let tweet = &mut ctx.accounts.tweet;
    let author = &ctx.accounts.reaction_author;

    match reaction {
        ReactionType::Like => tweet.likes = tweet.likes.saturating_add(1),
        ReactionType::Dislike => tweet.dislikes = tweet.dislikes.saturating_add(1),
    }

    tweet_reaction.reaction_author = author.key();
    tweet_reaction.parent_tweet = tweet.key();
    tweet_reaction.reaction = reaction;
    tweet_reaction.bump = ctx.bumps.tweet_reaction;

    Ok(())
}