use poise::{
    CreateReply, command,
    serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateMessage, Timestamp, User},
};
use rig_agent::AgentBuilder; //use rig_core::{client::completion::CompletionClient, completion::Prompt};

use crate::{
    ACCENT, Context, Error,
    util::{send_error, send_message, voice},
};

/// Ping The Yule Man
#[command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .color(ACCENT)
        .description(format!("Pong! 🏓 `{}`ms", ctx.ping().await.as_millis()))
        .timestamp(Timestamp::now());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Retrive your or another user's information
#[command(slash_command)]
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

/// Have The Yule Man join your current voice channel
#[command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    if let Some((manager, guild_id, channel_id)) = voice(ctx).await {
        send_message(
            ctx,
            format!(
                "Joined `{}`",
                channel_id
                    .name(ctx.http())
                    .await
                    .unwrap_or("unknown channel".into())
            ),
        )
        .await?;

        // This seems to hang so send the join message before this.
        manager.join(guild_id, channel_id).await?;
    }

    Ok(())
}

/// Have The Yule Man leave its voice channel
#[command(slash_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    if let Some((manager, guild_id, channel_id)) = voice(ctx).await {
        match manager.leave(guild_id).await {
            Ok(..) => {
                send_message(
                    ctx,
                    format!(
                        "Left `{}`",
                        channel_id
                            .name(ctx.http())
                            .await
                            .unwrap_or("unknown channel".into())
                    ),
                )
                .await?
            }
            Err(err) => send_error(ctx, format!("Failed to leave voice channel: {}", err)).await?,
        }
    }

    Ok(())
}

/// Tiki Tiki
#[command(slash_command)]
pub async fn tikitiki(ctx: Context<'_>) -> Result<(), Error> {
    if let Some((manager, guild_id, ..)) = voice(ctx).await {
        ctx.defer().await?;

        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;

            let sources_lock = ctx.data().sound_store.clone();
            let sources = sources_lock.lock().await;
            let source = sources
                .get("tikitiki")
                .expect("Handle placed into cache at startup.");

            let song = handler.play_input(source.into());
            // let _ = song.set_volume(1.0);
            // let _ = song.enable_loop();

            send_message(ctx, "Playing Tiki Tiki").await?;
        } else {
            send_error(
                ctx,
                "Failed to play Tiki Tiki. Hint: am I in a voice channel?",
            )
            .await?;
        }
    }

    Ok(())
}

/// Chat with The Yule Man
#[command(slash_command)]
pub async fn chat(
    ctx: Context<'_>,
    #[description = "Message to ask The Yule Man"] message: String,
) -> Result<(), Error> {
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
        Err(err) => send_error(ctx, format!("{}", err)).await?,
    };

    Ok(())
}
