use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId};

use crate::design::tokens::text_size;

const INTER_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/Inter-Medium.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");
const MONO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
const MONO_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Medium.ttf");
const MONO_SEMIBOLD: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-SemiBold.ttf");
const ICONS: &[u8] = include_bytes!("../../assets/fonts/Phosphor-Regular.ttf");

const INTER_REGULAR_NAME: &str = "Inter-Regular";
const INTER_MEDIUM_NAME: &str = "Inter-Medium";
const INTER_SEMIBOLD_NAME: &str = "Inter-SemiBold";
const MONO_REGULAR_NAME: &str = "JetBrainsMono-Regular";
const MONO_MEDIUM_NAME: &str = "JetBrainsMono-Medium";
const MONO_SEMIBOLD_NAME: &str = "JetBrainsMono-SemiBold";
const ICONS_NAME: &str = "Phosphor-Regular";

pub const UI_MEDIUM_FAMILY: &str = "ui-medium";
pub const UI_SEMIBOLD_FAMILY: &str = "ui-semibold";
pub const VALUE_MEDIUM_FAMILY: &str = "value-medium";
pub const VALUE_SEMIBOLD_FAMILY: &str = "value-semibold";
pub const ICON_FAMILY: &str = "icons";

pub fn font_definitions() -> FontDefinitions {
    let mut definitions = FontDefinitions::default();
    let fallback_proportional = definitions
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let fallback_monospace = definitions
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();

    for (name, bytes) in [
        (INTER_REGULAR_NAME, INTER_REGULAR),
        (INTER_MEDIUM_NAME, INTER_MEDIUM),
        (INTER_SEMIBOLD_NAME, INTER_SEMIBOLD),
        (MONO_REGULAR_NAME, MONO_REGULAR),
        (MONO_MEDIUM_NAME, MONO_MEDIUM),
        (MONO_SEMIBOLD_NAME, MONO_SEMIBOLD),
        (ICONS_NAME, ICONS),
    ] {
        definitions
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }

    let with_fallback = |primary: &str, fallback: &[String]| {
        std::iter::once(primary.to_owned())
            .chain(fallback.iter().cloned())
            .collect::<Vec<_>>()
    };
    definitions.families.insert(
        FontFamily::Proportional,
        with_fallback(INTER_REGULAR_NAME, &fallback_proportional),
    );
    definitions.families.insert(
        FontFamily::Monospace,
        with_fallback(MONO_REGULAR_NAME, &fallback_monospace),
    );
    for (family, primary, fallback) in [
        (UI_MEDIUM_FAMILY, INTER_MEDIUM_NAME, &fallback_proportional),
        (
            UI_SEMIBOLD_FAMILY,
            INTER_SEMIBOLD_NAME,
            &fallback_proportional,
        ),
        (VALUE_MEDIUM_FAMILY, MONO_MEDIUM_NAME, &fallback_monospace),
        (
            VALUE_SEMIBOLD_FAMILY,
            MONO_SEMIBOLD_NAME,
            &fallback_monospace,
        ),
    ] {
        definitions.families.insert(
            FontFamily::Name(family.into()),
            with_fallback(primary, fallback),
        );
    }
    definitions.families.insert(
        FontFamily::Name(ICON_FAMILY.into()),
        vec![ICONS_NAME.to_owned()],
    );
    definitions
}

pub fn install(context: &egui::Context) {
    context.set_fonts(font_definitions());
}

pub fn ui(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn ui_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(UI_MEDIUM_FAMILY.into()))
}

pub fn ui_semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(UI_SEMIBOLD_FAMILY.into()))
}

pub fn value(size: f32) -> FontId {
    FontId::monospace(size)
}

pub fn value_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(VALUE_MEDIUM_FAMILY.into()))
}

pub fn value_semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(VALUE_SEMIBOLD_FAMILY.into()))
}

pub fn caption() -> FontId {
    ui(text_size::CAPTION)
}

pub fn icon(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(ICON_FAMILY.into()))
}

pub fn label() -> FontId {
    ui_medium(text_size::LABEL)
}

pub fn body() -> FontId {
    ui(text_size::BODY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_family_has_a_primary_and_fallbacks() {
        let definitions = font_definitions();
        for family in [
            UI_MEDIUM_FAMILY,
            UI_SEMIBOLD_FAMILY,
            VALUE_MEDIUM_FAMILY,
            VALUE_SEMIBOLD_FAMILY,
        ] {
            let fonts = definitions
                .families
                .get(&FontFamily::Name(family.into()))
                .unwrap();
            assert!(fonts.len() > 1, "{family} has no fallback");
            assert!(definitions.font_data.contains_key(&fonts[0]));
        }
        assert_eq!(
            definitions.families[&FontFamily::Proportional][0],
            INTER_REGULAR_NAME
        );
        assert_eq!(
            definitions.families[&FontFamily::Monospace][0],
            MONO_REGULAR_NAME
        );
    }
}
