use std::collections::LinkedList;
use poise::CreateReply;
use crate::{Context, Error, TO_BE_QUEUED, VIEWABLE_QUEUE, serenity};

#[poise::command(slash_command, prefix_command)]
pub async fn prnt(ctx: Context<'_>, #[description = "Print something (powerful)"] text: String) -> Result<(), Error> {
    println!("{}", text);
    ctx.say("Printed your request".to_string()).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn get_queue(ctx: Context<'_>) -> Result<(), Error> {
    let cloned_queue: LinkedList<String> = VIEWABLE_QUEUE.clone().read().await.clone();
    ctx.say(if cloned_queue.is_empty() {"Queue is currently empty.".to_string()} else {format!("**Current Queue ({:?}):** `{:?}`", cloned_queue.len(), &cloned_queue)}).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn request(ctx: Context<'_>) -> Result<(), Error> {
    if VIEWABLE_QUEUE.read().await.iter().any(|id| id.to_owned() == ctx.author().id.to_string()) {
        ctx.send(
            CreateReply::default()
                .embed(serenity::CreateEmbed::default().description("You are already in the queue, please wait."))
                .ephemeral(true)
        ).await?;
        Ok(())
    } else {
        let reply = ctx.send(
            CreateReply::default()
                .embed(serenity::CreateEmbed::default().description("You have joined the queue, please wait."))
                .ephemeral(false)
        ).await?;
        TO_BE_QUEUED.write().await.push(ctx.author().id.to_string());
        loop {
            reply.edit(ctx,
                CreateReply::default()
                    .embed(serenity::CreateEmbed::default().description("This is an edited message..."))
            ).await?;
        }
        Ok(())
    }
}