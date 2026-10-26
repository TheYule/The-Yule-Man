use std::sync::Arc;

use poise::{
    CreateReply,
    serenity_prelude::{ChannelId, Colour, CreateEmbed, GuildId, async_trait},
};
use songbird::{Event, EventContext, Songbird, events::EventHandler as VoiceEventHandler};

use crate::{ACCENT, Context, Error};

pub fn basic_embed(message: impl Into<String>, color: Colour) -> CreateEmbed {
    CreateEmbed::new().color(color).description(message)
}

pub async fn send_message(ctx: Context<'_>, message: impl Into<String>) -> Result<(), Error> {
    let embed = basic_embed(format!("{}", message.into()), ACCENT);
    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

pub async fn send_error(ctx: Context<'_>, message: impl Into<String>) -> Result<(), Error> {
    let embed = basic_embed(message, Colour::RED).title("Error");
    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

pub async fn voice(ctx: Context<'_>) -> Option<(Arc<Songbird>, GuildId, ChannelId)> {
    let guild_id = ctx.guild_id().unwrap();
    let voice_state = guild_id
        .get_user_voice_state(ctx.http(), ctx.author().id)
        .await;
    let channel_id = match voice_state {
        Ok(state) => state.channel_id,
        Err(..) => None,
    };

    let connect_to = match channel_id {
        Some(id) => id,
        None => {
            send_error(ctx, "Please join a voice channel.")
                .await
                .unwrap();
            return None;
        }
    };

    match songbird::get(ctx.serenity_context()).await {
        Some(manager) => Some((manager, guild_id, connect_to)),
        None => {
            send_error(ctx, "Failed to retrieve voice client.")
                .await
                .unwrap();
            return None;
        }
    }
}

pub struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}
