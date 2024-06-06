use std::{fmt::Display, str::FromStr};

use image::Rgba;

/// A wrapper around Rgba<u8> to provide some extra functionality which helps for client libraries.
#[derive(Debug, Clone)]
pub struct Colour(pub Rgba<u8>);

impl Colour {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Colour(Rgba([r, g, b, a]))
    }

    pub fn new_from_arr(arr: [u8; 4]) -> Self {
        Colour(Rgba(arr))
    }
}

impl Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "rgba({},{},{},{})",
            self.0 .0[0], self.0 .0[1], self.0 .0[2], self.0 .0[3],
        ))
    }
}

impl From<Colour> for Rgba<u8> {
    fn from(val: Colour) -> Self {
        val.0
    }
}

#[cfg(feature = "parse")]
impl FromStr for Colour {
    type Err = csscolorparser::ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        csscolorparser::Color::from_str(s).map(|x| Colour::new_from_arr(x.to_rgba8()))
    }
}

#[cfg(feature = "serenity")]
#[async_trait::async_trait]
impl poise::SlashArgument for Colour {
    async fn extract(
        _ctx: &poise::serenity_prelude::Context,
        _interaction: &poise::serenity_prelude::CommandInteraction,
        value: &poise::serenity_prelude::ResolvedValue<'_>,
    ) -> Result<Self, poise::SlashArgError> {
        match *value {
            poise::serenity_prelude::ResolvedValue::String(x) => {
                Colour::from_str(x).map_err(|_| {
                    poise::SlashArgError::new_command_structure_mismatch("could not parse colour")
                })
            }

            _ => Err(poise::SlashArgError::new_command_structure_mismatch(
                "expected string",
            )),
        }
    }

    fn create(
        builder: poise::serenity_prelude::CreateCommandOption,
    ) -> poise::serenity_prelude::CreateCommandOption {
        builder.kind(poise::serenity_prelude::CommandOptionType::String)
    }
}
