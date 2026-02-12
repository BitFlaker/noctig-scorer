use std::sync::LazyLock;
use std::collections::HashMap;

use crate::external::scientific_colour_maps;

pub type ColorMap = LazyLock<HashMap<String, Vec<[f32; 4]>>>;

/// A list of all available color map sources
pub const COLOR_MAPS: LazyLock<Vec<&ColorMap>> = LazyLock::new(|| {
    vec![
        &scientific_colour_maps::COLOR_MAPS
    ]
});

/// Tries to get the specified color gradient from all available color map sources.
/// If nothing is found under the specified name, a white color is returned.
/// By appending `_r` to the name, the resulting color gradient will be reversed.
pub fn get_color_map(name: &String) -> Vec<[f32; 4]> {
    // Get the name and check if the gradient should be reversed
    let name_stripped = name.strip_suffix("_r");
    let name = name_stripped.unwrap_or(name);
    let reverse = name_stripped.is_some();

    // Get the gradient colors and reverse it if requested
    for map in COLOR_MAPS.iter() {
        if let Some(mut colors) = map.get(name).cloned() {
            if reverse {
                colors.reverse();
            }
            return colors;
        }
    }

    // Default to white if no color map was found with that name
    vec![[1.0, 1.0, 1.0, 1.0]]
}
