use poise::{
    CreateReply,
    serenity_prelude::{Colour, CreateEmbed},
};

use crate::{ACCENT, Context, Error};

pub fn basic_embed(message: impl Into<String>, color: Colour) -> CreateEmbed {
    CreateEmbed::new().color(color).description(message)
}

pub async fn send_message(ctx: Context<'_>, message: impl Into<String>) -> Result<(), Error> {
    let embed = basic_embed(message.into(), ACCENT);
    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

pub async fn send_error(ctx: Context<'_>, message: impl Into<String>) -> Result<(), Error> {
    let embed = basic_embed(message, Colour::RED).title("Error");
    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
