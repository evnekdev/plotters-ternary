//! Shared fixed-slot geometry for native Plotters legends in examples.

use crate::output_support::scaled;

pub(crate) const LEGEND_SYMBOL_SLOT_WIDTH: u32 = 34;
pub(crate) const LEGEND_SYMBOL_LABEL_GAP: u32 = 12;
pub(crate) const LEGEND_OUTER_PADDING: u32 = 12;
pub(crate) const LEGEND_TEXT_SIZE: u32 = 22;
// Plotters supplies the floor of an odd-height label box midpoint to legend closures.
const LEGEND_TEXT_CENTRE_CEILING_CORRECTION: u32 = 1;
// The fitted high-resolution Plotters layout rounds its origin per scale step.
const LEGEND_SUPERSAMPLED_X_ORIGIN_ROUNDING: i32 = 5;
// Plotters' fitted high-resolution layout has a stable non-integral Y-origin
// offset. This conversion maps its integer callback anchors back to the
// final-resolution legend-row centres for the supported 2x, 3x, and 4x modes.
const LEGEND_SUPERSAMPLED_Y_ORIGIN_ROUNDING_NUMERATOR: u32 = 17;
const LEGEND_SUPERSAMPLED_Y_ORIGIN_ROUNDING_DENOMINATOR: u32 = 3;

/// The shared physical layout for one Plotters legend row.
///
/// Plotters supplies the left edge of its legend area to a `SeriesAnno`
/// closure. This adapter reserves a fixed symbol slot at that edge, then
/// supplies its centre to every built-in and custom symbol renderer. Text
/// starts after the slot and a fixed gap, which is the same coordinate that
/// Plotters uses after `legend_area_size` is configured below.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LegendRowLayout {
    pub(crate) row_center_y: i32,
    pub(crate) symbol_center_x: i32,
    pub(crate) symbol_slot_width: u32,
    pub(crate) label_start_x: i32,
}

impl LegendRowLayout {
    pub(crate) fn from_plotters_anchor(anchor: (i32, i32), scale: u32) -> Self {
        let symbol_slot_width = scaled(LEGEND_SYMBOL_SLOT_WIDTH, scale);
        let label_gap = scaled(LEGEND_SYMBOL_LABEL_GAP, scale);
        let scale_steps = scale.saturating_sub(1);
        let x_origin_correction =
            LEGEND_SUPERSAMPLED_X_ORIGIN_ROUNDING.saturating_mul(scale_steps as i32);
        let y_center_correction = LEGEND_TEXT_CENTRE_CEILING_CORRECTION.saturating_add(
            LEGEND_SUPERSAMPLED_Y_ORIGIN_ROUNDING_NUMERATOR
                .saturating_mul(scale_steps)
                .saturating_add(1)
                / LEGEND_SUPERSAMPLED_Y_ORIGIN_ROUNDING_DENOMINATOR,
        ) as i32;
        let symbol_slot_left_x = anchor.0 - x_origin_correction;
        Self {
            row_center_y: anchor.1 + y_center_correction,
            symbol_center_x: symbol_slot_left_x + symbol_slot_width as i32 / 2,
            symbol_slot_width,
            label_start_x: symbol_slot_left_x + symbol_slot_width as i32 + label_gap as i32,
        }
    }

    pub(crate) const fn symbol_center(self) -> (i32, i32) {
        (self.symbol_center_x, self.row_center_y)
    }

    pub(crate) fn line_endpoints(self) -> ((i32, i32), (i32, i32)) {
        let half_width = self.symbol_slot_width as i32 / 2;
        (
            (self.symbol_center_x - half_width, self.row_center_y),
            (self.symbol_center_x + half_width, self.row_center_y),
        )
    }

    /// Call a custom legend-symbol closure with the centre of its symbol slot.
    pub(crate) fn custom_symbol<E, F>(self, make_symbol: F) -> E
    where
        F: FnOnce((i32, i32)) -> E,
    {
        make_symbol(self.symbol_center())
    }
}
