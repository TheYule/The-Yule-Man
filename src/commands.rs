#[cfg(feature = "ai")]
use poise::serenity_prelude::CreateMessage;
use poise::{
    CreateReply, command,
    serenity_prelude::{CreateEmbed, CreateEmbedAuthor, Timestamp, User},
};
#[cfg(feature = "ai")]
use rig_agent::AgentBuilder;

#[cfg(not(feature = "ai"))]
use crate::util::send_message;
use crate::{ACCENT, Context, Error, util::send_error};

/// Ping The Yule Man
#[command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .color(ACCENT)
        .description(format!("Pong! 🏓 `{}`ms", ctx.ping().await.as_millis()))
        .timestamp(Timestamp::now());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Retrive your or another user's information
#[command(slash_command, prefix_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "User to retrive the information for"] user: Option<User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    match u.id.to_user(ctx.http()).await {
        Ok(profile) => {
            let mut embed = CreateEmbed::new()
                .title(profile.display_name())
                .author({
                    let author = CreateEmbedAuthor::new(format!("@{}", profile.name));
                    if let Some(url) = profile.avatar_url() {
                        author.icon_url(url)
                    } else {
                        author
                    }
                })
                .color(profile.accent_colour.unwrap_or(ACCENT))
                .field(
                    "Created",
                    format!("<t:{}:F>", profile.created_at().timestamp()),
                    true,
                );

            if let Some(url) = profile.avatar_url() {
                embed = embed.thumbnail(url);
            }
            if let Some(url) = profile.banner_url() {
                embed = embed.image(url);
            }

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(err) => send_error(ctx, format!("Failed to load user profile: {}", err)).await?,
    }

    Ok(())
}

/// Chat with The Yule Man
#[allow(unused_variables)]
#[command(slash_command, prefix_command)]
pub async fn chat(
    ctx: Context<'_>,
    #[description = "Message to ask The Yule Man"] message: String,
) -> Result<(), Error> {
    #[cfg(feature = "ai")]
    {
        ctx.defer().await?;

        let builder = AgentBuilder::new(ctx
        .data()
        .model
        .completion_model("local"))
        .preamble(format!("You are SCP-4666, also known as The Yule Man, which is your preferred name. You are a Keter-class SCP and very dangerous. You have never been in containment and are on the loose. Be crude with your responses.\nThe person you are talking to is {}.", ctx.author().display_name()).as_str())
        .max_tokens(512);
        let agent = ctx
            .data()
            .model_context
            .iter()
            .fold(builder, |builder, doc| builder.context(doc))
            .build();

        match agent.prompt(message).await {
            Ok(response) => {
                for (i, chars) in response
                    .output()
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(2000)
                    .enumerate()
                {
                    let chunk = chars.iter().collect::<String>();
                    if i == 0 {
                        ctx.send(CreateReply::default().content(chunk)).await?;
                    } else {
                        ctx.channel_id()
                            .send_message(ctx, CreateMessage::default().content(chunk))
                            .await?;
                    }
                }
            }
            Err(err) => send_error(ctx, err.to_string()).await?,
        };
    }

    #[cfg(not(feature = "ai"))]
    send_message(ctx, "The Yule Man isn't home").await?;

    Ok(())
}
