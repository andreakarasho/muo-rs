//! Grid layout (`GridSizeSpec`, `GridCell`, `GridEntryStyle`, `GridCalculator`, the grid builder and entry
//! extensions, `ListViewLayout`): grid and list layout arithmetic for the dynamic
//! gump builder. Spans are slices; the interpolated-string overloads are the `&str` ones
//! (callers `format!`).

use super::builder::{GumpBuilder, HtmlStyle};
use super::layout::GumpButtonType;

// ---- GridSizeSpec ----

/// `GridSizeType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GridSizeType {
    Absolute,
    Star,
    Percent,
}

/// `GridSizeSpec`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GridSizeSpec {
    pub kind: GridSizeType,
    pub value: i32,
}

impl GridSizeSpec {
    pub fn absolute(pixels: i32) -> Self {
        Self {
            kind: GridSizeType::Absolute,
            value: pixels,
        }
    }

    pub fn star() -> Self {
        Self {
            kind: GridSizeType::Star,
            value: 1,
        }
    }

    pub fn percent(percent: i32) -> Self {
        Self {
            kind: GridSizeType::Percent,
            value: percent,
        }
    }

    /// `Parse(token)`: `100` pixels, `*` an equal share, `10*` a percentage; anything unparseable is a star.
    pub fn parse(token: &str) -> Self {
        let token = token.trim();
        if token.is_empty() {
            return Self::star();
        }
        match token.find('*') {
            None => token.parse().map_or_else(|_| Self::star(), Self::absolute),
            Some(0) if token.len() == 1 => Self::star(),
            Some(i) => token[..i]
                .parse()
                .map_or_else(|_| Self::star(), Self::percent),
        }
    }

    /// `ParseAll(sizeSpec, destination)`: tokens separated by spaces, at most `destination.len()` of them.
    pub fn parse_all(size_spec: &str, destination: &mut [GridSizeSpec]) -> usize {
        let mut count = 0;
        let mut remaining = size_spec;
        while !remaining.is_empty() && count < destination.len() {
            remaining = remaining.trim_start();
            if remaining.is_empty() {
                break;
            }
            let token = match remaining.find(' ') {
                None => std::mem::take(&mut remaining),
                Some(i) => {
                    let t = &remaining[..i];
                    remaining = &remaining[i + 1..];
                    t
                }
            };
            destination[count] = Self::parse(token);
            count += 1;
        }
        count
    }
}

// ---- GridCell ----

/// `GridCell`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl GridCell {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    pub fn center_x(&self) -> i32 {
        self.x + self.width / 2
    }

    pub fn center_y(&self) -> i32 {
        self.y + self.height / 2
    }

    /// `Inset(margin)`.
    pub fn inset(&self, margin: i32) -> Self {
        Self::new(
            self.x + margin,
            self.y + margin,
            self.width - margin * 2,
            self.height - margin * 2,
        )
    }

    /// `Inset(horizontal, vertical)`.
    pub fn inset_hv(&self, horizontal: i32, vertical: i32) -> Self {
        Self::new(
            self.x + horizontal,
            self.y + vertical,
            self.width - horizontal * 2,
            self.height - vertical * 2,
        )
    }

    /// `Inset(left, top, right, bottom)`.
    pub fn inset_ltrb(&self, left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self::new(
            self.x + left,
            self.y + top,
            self.width - left - right,
            self.height - top - bottom,
        )
    }

    pub fn offset(&self, offset_x: i32, offset_y: i32) -> Self {
        Self::new(
            self.x + offset_x,
            self.y + offset_y,
            self.width,
            self.height,
        )
    }

    pub fn with_width(&self, width: i32) -> Self {
        Self::new(self.x, self.y, width, self.height)
    }

    pub fn with_height(&self, height: i32) -> Self {
        Self::new(self.x, self.y, self.width, height)
    }
}

// ---- GridEntryStyle ----

/// `GridEntryStyle`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GridEntryStyle {
    pub entry_gump_id: i32,
    pub header_gump_id: i32,
    pub offset_gump_id: i32,
    pub back_gump_id: i32,
    pub text_hue: i32,
    pub text_offset_x: i32,
    pub entry_height: i32,
    pub border_size: i32,
    pub offset_size: i32,
}

impl GridEntryStyle {
    pub const ARROW_LEFT_ID1: i32 = 0x15E3;
    pub const ARROW_LEFT_ID2: i32 = 0x15E7;
    pub const ARROW_RIGHT_ID1: i32 = 0x15E1;
    pub const ARROW_RIGHT_ID2: i32 = 0x15E5;
    pub const ARROW_WIDTH: i32 = 16;
    pub const ARROW_HEIGHT: i32 = 16;

    /// `Default`.
    pub const DEFAULT: GridEntryStyle = GridEntryStyle {
        entry_gump_id: 0x0BBC,
        header_gump_id: 0x0E14,
        offset_gump_id: 0x0A40,
        back_gump_id: 0x13BE,
        text_hue: 0,
        text_offset_x: 2,
        entry_height: 20,
        border_size: 10,
        offset_size: 1,
    };

    /// `With(..)`: a copy with the given (non-negative) values replaced; pass -1 to keep one.
    #[allow(clippy::too_many_arguments)]
    pub fn with(
        &self,
        entry_gump_id: i32,
        header_gump_id: i32,
        offset_gump_id: i32,
        back_gump_id: i32,
        text_hue: i32,
        text_offset_x: i32,
        entry_height: i32,
        border_size: i32,
        offset_size: i32,
    ) -> Self {
        let pick = |new: i32, old: i32| if new < 0 { old } else { new };
        Self {
            entry_gump_id: pick(entry_gump_id, self.entry_gump_id),
            header_gump_id: pick(header_gump_id, self.header_gump_id),
            offset_gump_id: pick(offset_gump_id, self.offset_gump_id),
            back_gump_id: pick(back_gump_id, self.back_gump_id),
            text_hue: pick(text_hue, self.text_hue),
            text_offset_x: pick(text_offset_x, self.text_offset_x),
            entry_height: pick(entry_height, self.entry_height),
            border_size: pick(border_size, self.border_size),
            offset_size: pick(offset_size, self.offset_size),
        }
    }

    pub fn content_origin_x(&self) -> i32 {
        self.border_size + self.offset_size
    }

    pub fn content_origin_y(&self) -> i32 {
        self.border_size + self.offset_size
    }

    pub fn total_width(&self, content_width: i32) -> i32 {
        content_width + self.border_size * 2 + self.offset_size * 2
    }

    pub fn total_height(&self, row_count: i32) -> i32 {
        self.border_size * 2
            + self.offset_size * 2
            + row_count * self.entry_height
            + (row_count - 1) * self.offset_size
    }

    pub fn content_height(&self, row_count: i32) -> i32 {
        row_count * self.entry_height + (row_count - 1) * self.offset_size
    }
}

// ---- GridCalculator ----

/// `GridCalculator.MaxTracks`.
pub const MAX_TRACKS: usize = 32;

/// `ComputeTrackSizes(specs, totalSize, origin, positions, sizes)`.
pub fn compute_track_sizes(
    specs: &[GridSizeSpec],
    total_size: i32,
    origin: i32,
    positions: &mut [i32],
    sizes: &mut [i32],
) {
    let track_count = specs.len();
    // First pass: calculate absolute/percent sizes and count star tracks
    let mut remaining_size = total_size;
    let mut star_count = 0;
    for (i, spec) in specs.iter().enumerate() {
        match spec.kind {
            GridSizeType::Absolute => {
                sizes[i] = spec.value;
                remaining_size -= spec.value;
            }
            GridSizeType::Percent => {
                sizes[i] = total_size.wrapping_mul(spec.value) / 100;
                remaining_size -= sizes[i];
            }
            GridSizeType::Star => {
                star_count += 1;
                sizes[i] = -1; // Mark as star for second pass
            }
        }
    }
    // Second pass: distribute remaining space to star tracks
    if star_count > 0 && remaining_size > 0 {
        let star_size = remaining_size / star_count;
        for s in sizes.iter_mut().take(track_count) {
            if *s == -1 {
                *s = star_size;
            }
        }
    } else {
        // No star tracks or no remaining space - set any marked stars to 0
        for s in sizes.iter_mut().take(track_count) {
            if *s < 0 {
                *s = 0;
            }
        }
    }
    // Third pass: calculate positions
    let mut pos = origin;
    for i in 0..track_count {
        positions[i] = pos;
        pos += sizes[i];
    }
}

/// `ComputeUniformTrackSizes(trackCount, totalSize, origin, positions, sizes)`.
pub fn compute_uniform_track_sizes(
    track_count: usize,
    total_size: i32,
    origin: i32,
    positions: &mut [i32],
    sizes: &mut [i32],
) {
    let track_size = total_size / track_count as i32; // panics for 0
    let mut pos = origin;
    for i in 0..track_count {
        positions[i] = pos;
        sizes[i] = track_size;
        pos += track_size;
    }
}

/// `ComputeFromSpec(sizeSpec, totalSize, origin, positions, sizes)`: the number of tracks.
pub fn compute_from_spec(
    size_spec: &str,
    total_size: i32,
    origin: i32,
    positions: &mut [i32],
    sizes: &mut [i32],
) -> usize {
    compute_from_spec_gap(size_spec, total_size, origin, 0, positions, sizes)
}

/// `ComputeFromSpec(sizeSpec, totalSize, origin, gap, positions, sizes)`.
pub fn compute_from_spec_gap(
    size_spec: &str,
    total_size: i32,
    origin: i32,
    gap: i32,
    positions: &mut [i32],
    sizes: &mut [i32],
) -> usize {
    let mut specs = [GridSizeSpec::star(); MAX_TRACKS];
    let track_count = GridSizeSpec::parse_all(size_spec, &mut specs);
    if track_count > 0 {
        // Subtract total gap space from available size before computing track sizes
        let total_gap_space = gap * (track_count as i32 - 1);
        let available_size = total_size - total_gap_space;
        compute_track_sizes(
            &specs[..track_count],
            available_size,
            origin,
            positions,
            sizes,
        );
        // Adjust positions to account for gaps
        if gap > 0 {
            for (i, p) in positions.iter_mut().enumerate().take(track_count).skip(1) {
                *p += gap * i as i32;
            }
        }
    }
    track_count
}

/// `GetCell(columnPositions, columnWidths, rowPositions, rowHeights, column, row)`.
pub fn get_cell(
    column_positions: &[i32],
    column_widths: &[i32],
    row_positions: &[i32],
    row_heights: &[i32],
    column: usize,
    row: usize,
) -> GridCell {
    GridCell::new(
        column_positions[column],
        row_positions[row],
        column_widths[column],
        row_heights[row],
    )
}

/// `GetCell(.., column, row, columnSpan, rowSpan)`.
#[allow(clippy::too_many_arguments)] // one argument per layout parameter
pub fn get_cell_span(
    column_positions: &[i32],
    column_widths: &[i32],
    row_positions: &[i32],
    row_heights: &[i32],
    column: usize,
    row: usize,
    column_span: usize,
    row_span: usize,
) -> GridCell {
    let end_col = (column + column_span).min(column_widths.len());
    let width = column_widths
        .get(column..end_col)
        .map_or(0, |s| s.iter().sum());
    let end_row = (row + row_span).min(row_heights.len());
    let height = row_heights.get(row..end_row).map_or(0, |s| s.iter().sum());
    GridCell::new(column_positions[column], row_positions[row], width, height)
}

// ---- ListViewLayout ----

/// `ListViewLayout`.
#[derive(Copy, Clone, Debug)]
pub struct ListViewLayout {
    origin_y: i32,
    header_height: i32,
    row_height: i32,
    pub total_items: i32,
    pub items_per_page: i32,
    pub current_page: i32,
    pub total_pages: i32,
    pub start_index: i32,
    pub visible_count: i32,
    pub can_go_back: bool,
    pub can_go_next: bool,
    pub column_count: usize,
}

impl ListViewLayout {
    pub fn row_height(&self) -> i32 {
        self.row_height
    }

    pub fn header_height(&self) -> i32 {
        self.header_height
    }

    /// `Create(..)`: parses the column spec into `column_positions`/`column_widths` and lays out the pages.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        origin_x: i32,
        origin_y: i32,
        width: i32,
        height: i32,
        total_items: i32,
        current_page: i32,
        row_height: i32,
        header_height: i32,
        column_spec: &str,
        column_positions: &mut [i32],
        column_widths: &mut [i32],
    ) -> Self {
        let column_count = compute_from_spec(
            column_spec,
            width,
            origin_x,
            column_positions,
            column_widths,
        );
        // Calculate items per page based on available height
        let content_height = height - header_height;
        let items_per_page = content_height / row_height; // panics for 0
        let mut l = Self {
            origin_y,
            header_height,
            row_height,
            total_items,
            items_per_page,
            current_page: 0,
            total_pages: 1,
            start_index: 0,
            visible_count: 0,
            can_go_back: false,
            can_go_next: false,
            column_count,
        };
        // Calculate pagination
        if total_items > 0 && items_per_page > 0 {
            l.total_pages = (total_items + items_per_page - 1) / items_per_page;
            l.current_page = current_page.clamp(0, l.total_pages - 1);
            l.start_index = l.current_page * items_per_page;
            l.visible_count = items_per_page.min(total_items - l.start_index);
            l.can_go_back = l.current_page > 0;
            l.can_go_next = l.current_page < l.total_pages - 1;
        }
        l
    }

    pub fn get_header_cell(
        &self,
        column: usize,
        column_positions: &[i32],
        column_widths: &[i32],
    ) -> GridCell {
        GridCell::new(
            column_positions[column],
            self.origin_y,
            column_widths[column],
            self.header_height,
        )
    }

    pub fn get_row_cell(
        &self,
        visible_row_index: i32,
        column: usize,
        column_positions: &[i32],
        column_widths: &[i32],
    ) -> GridCell {
        GridCell::new(
            column_positions[column],
            self.origin_y + self.header_height + visible_row_index * self.row_height,
            column_widths[column],
            self.row_height,
        )
    }

    pub fn get_data_index(&self, visible_row_index: i32) -> i32 {
        self.start_index + visible_row_index
    }

    pub fn get_row_y(&self, visible_row_index: i32) -> i32 {
        self.origin_y + self.header_height + visible_row_index * self.row_height
    }
}

pub fn get_column_x(column_positions: &[i32], column: usize) -> i32 {
    column_positions[column]
}

pub fn get_column_width(column_widths: &[i32], column: usize) -> i32 {
    column_widths[column]
}

pub fn get_column_center_x(column_positions: &[i32], column_widths: &[i32], column: usize) -> i32 {
    column_positions[column] + column_widths[column] / 2
}

pub fn get_column_right_x(column_positions: &[i32], column_widths: &[i32], column: usize) -> i32 {
    column_positions[column] + column_widths[column]
}

// ---- grid builder and grid entry extensions ----

/// The `GridCell` overloads of `DynamicGumpBuilder`.
pub trait GridBuilderExt {
    fn add_background_cell(&mut self, cell: &GridCell, gump_id: i32);
    fn add_alpha_region_cell(&mut self, cell: &GridCell);
    fn add_image_tiled_cell(&mut self, cell: &GridCell, gump_id: i32);
    fn add_image_cell(
        &mut self,
        cell: &GridCell,
        gump_id: i32,
        hue: i32,
        offset_x: i32,
        offset_y: i32,
    );
    fn add_item_cell(
        &mut self,
        cell: &GridCell,
        item_id: i32,
        hue: i32,
        offset_x: i32,
        offset_y: i32,
    );
    fn add_label_cell(
        &mut self,
        cell: &GridCell,
        hue: i32,
        text: &str,
        offset_x: i32,
        offset_y: i32,
    );
    fn add_label_cropped_cell(
        &mut self,
        cell: &GridCell,
        hue: i32,
        text: &str,
        offset_x: i32,
        offset_y: i32,
    );
    fn add_html_cell(
        &mut self,
        cell: &GridCell,
        text: &str,
        style: HtmlStyle<'_>,
        background: bool,
        scrollbar: bool,
    );
    #[allow(clippy::too_many_arguments)] // one argument per layout parameter
    fn add_html_cell_offset(
        &mut self,
        cell: &GridCell,
        text: &str,
        offset_x: i32,
        offset_y: i32,
        style: HtmlStyle<'_>,
        background: bool,
        scrollbar: bool,
    );
    fn add_html_localized_cell(
        &mut self,
        cell: &GridCell,
        number: i32,
        background: bool,
        scrollbar: bool,
    );
    fn add_html_localized_color_cell(
        &mut self,
        cell: &GridCell,
        number: i32,
        color: i32,
        background: bool,
        scrollbar: bool,
    );
    fn add_html_localized_args_cell(
        &mut self,
        cell: &GridCell,
        number: i32,
        args: &str,
        color: i32,
        background: bool,
        scrollbar: bool,
    );
    #[allow(clippy::too_many_arguments)]
    fn add_button_cell(
        &mut self,
        cell: &GridCell,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        kind: GumpButtonType,
        param: i32,
        offset_x: i32,
        offset_y: i32,
    );
    #[allow(clippy::too_many_arguments)]
    fn add_checkbox_cell(
        &mut self,
        cell: &GridCell,
        inactive_id: i32,
        active_id: i32,
        selected: bool,
        switch_id: i32,
        offset_x: i32,
        offset_y: i32,
    );
    #[allow(clippy::too_many_arguments)]
    fn add_radio_cell(
        &mut self,
        cell: &GridCell,
        inactive_id: i32,
        active_id: i32,
        selected: bool,
        switch_id: i32,
        offset_x: i32,
        offset_y: i32,
    );
    fn add_text_entry_cell(&mut self, cell: &GridCell, hue: i32, entry_id: i32, initial_text: &str);
    fn add_text_entry_cell_offset(
        &mut self,
        cell: &GridCell,
        hue: i32,
        entry_id: i32,
        offset_x: i32,
        offset_y: i32,
        initial_text: &str,
    );
    fn add_text_entry_limited_cell(
        &mut self,
        cell: &GridCell,
        hue: i32,
        entry_id: i32,
        size: i32,
        initial_text: &str,
    );
    #[allow(clippy::too_many_arguments)]
    fn add_image_tiled_button_cell(
        &mut self,
        cell: &GridCell,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        kind: GumpButtonType,
        param: i32,
        item_id: i32,
        hue: i32,
        localized_tooltip: i32,
    );
    fn add_sprite_image_cell(&mut self, cell: &GridCell, gump_id: i32, sx: i32, sy: i32);
    #[allow(clippy::too_many_arguments)]
    fn add_sprite_image_cell_sized(
        &mut self,
        cell: &GridCell,
        gump_id: i32,
        width: i32,
        height: i32,
        sx: i32,
        sy: i32,
        offset_x: i32,
        offset_y: i32,
    );

    // GridEntryExtensions
    fn add_entry_label(&mut self, cell: &GridCell, style: &GridEntryStyle, text: &str);
    fn add_entry_html(&mut self, cell: &GridCell, style: &GridEntryStyle, text: &str);
    fn add_entry_header(&mut self, cell: &GridCell, style: &GridEntryStyle);
    #[allow(clippy::too_many_arguments)]
    fn add_entry_button(
        &mut self,
        cell: &GridCell,
        style: &GridEntryStyle,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        button_width: i32,
        button_height: i32,
    );
    fn add_entry_arrow_left(&mut self, cell: &GridCell, style: &GridEntryStyle, button_id: i32);
    fn add_entry_arrow_right(&mut self, cell: &GridCell, style: &GridEntryStyle, button_id: i32);
    fn add_entry_text(
        &mut self,
        cell: &GridCell,
        style: &GridEntryStyle,
        entry_id: i32,
        initial_text: &str,
    );
    fn add_entry_blank(&mut self, cell: &GridCell, style: &GridEntryStyle);
    fn add_grid_background(&mut self, width: i32, height: i32, style: &GridEntryStyle);
}

impl GridBuilderExt for GumpBuilder {
    fn add_background_cell(&mut self, c: &GridCell, gump_id: i32) {
        self.add_background(c.x, c.y, c.width, c.height, gump_id);
    }

    fn add_alpha_region_cell(&mut self, c: &GridCell) {
        self.add_alpha_region(c.x, c.y, c.width, c.height);
    }

    fn add_image_tiled_cell(&mut self, c: &GridCell, gump_id: i32) {
        self.add_image_tiled(c.x, c.y, c.width, c.height, gump_id);
    }

    fn add_image_cell(&mut self, c: &GridCell, gump_id: i32, hue: i32, ox: i32, oy: i32) {
        self.add_image(c.x + ox, c.y + oy, gump_id, hue);
    }

    fn add_item_cell(&mut self, c: &GridCell, item_id: i32, hue: i32, ox: i32, oy: i32) {
        self.add_item(c.x + ox, c.y + oy, item_id, hue);
    }

    fn add_label_cell(&mut self, c: &GridCell, hue: i32, text: &str, ox: i32, oy: i32) {
        self.add_label(c.x + ox, c.y + oy, hue, text);
    }

    fn add_label_cropped_cell(&mut self, c: &GridCell, hue: i32, text: &str, ox: i32, oy: i32) {
        self.add_label_cropped(c.x + ox, c.y + oy, c.width - ox, c.height - oy, hue, text);
    }

    fn add_html_cell(
        &mut self,
        c: &GridCell,
        text: &str,
        style: HtmlStyle<'_>,
        background: bool,
        scrollbar: bool,
    ) {
        self.add_html_styled(
            c.x, c.y, c.width, c.height, text, style, background, scrollbar,
        );
    }

    fn add_html_cell_offset(
        &mut self,
        c: &GridCell,
        text: &str,
        ox: i32,
        oy: i32,
        style: HtmlStyle<'_>,
        background: bool,
        scrollbar: bool,
    ) {
        self.add_html_styled(
            c.x + ox,
            c.y + oy,
            c.width - ox,
            c.height - oy,
            text,
            style,
            background,
            scrollbar,
        );
    }

    fn add_html_localized_cell(
        &mut self,
        c: &GridCell,
        number: i32,
        background: bool,
        scrollbar: bool,
    ) {
        self.add_html_localized(c.x, c.y, c.width, c.height, number, background, scrollbar);
    }

    fn add_html_localized_color_cell(
        &mut self,
        c: &GridCell,
        number: i32,
        color: i32,
        background: bool,
        scrollbar: bool,
    ) {
        self.add_html_localized_color(
            c.x, c.y, c.width, c.height, number, color, background, scrollbar,
        );
    }

    fn add_html_localized_args_cell(
        &mut self,
        c: &GridCell,
        number: i32,
        args: &str,
        color: i32,
        background: bool,
        scrollbar: bool,
    ) {
        self.add_html_localized_args(
            c.x, c.y, c.width, c.height, number, args, color, background, scrollbar,
        );
    }

    fn add_button_cell(
        &mut self,
        c: &GridCell,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        kind: GumpButtonType,
        param: i32,
        ox: i32,
        oy: i32,
    ) {
        self.add_button_type(
            c.x + ox,
            c.y + oy,
            normal_id,
            pressed_id,
            button_id,
            kind,
            param,
        );
    }

    fn add_checkbox_cell(
        &mut self,
        c: &GridCell,
        inactive_id: i32,
        active_id: i32,
        selected: bool,
        switch_id: i32,
        ox: i32,
        oy: i32,
    ) {
        self.add_checkbox(
            c.x + ox,
            c.y + oy,
            inactive_id,
            active_id,
            selected,
            switch_id,
        );
    }

    fn add_radio_cell(
        &mut self,
        c: &GridCell,
        inactive_id: i32,
        active_id: i32,
        selected: bool,
        switch_id: i32,
        ox: i32,
        oy: i32,
    ) {
        self.add_radio(
            c.x + ox,
            c.y + oy,
            inactive_id,
            active_id,
            selected,
            switch_id,
        );
    }

    fn add_text_entry_cell(&mut self, c: &GridCell, hue: i32, entry_id: i32, initial_text: &str) {
        self.add_text_entry(c.x, c.y, c.width, c.height, hue, entry_id, initial_text);
    }

    fn add_text_entry_cell_offset(
        &mut self,
        c: &GridCell,
        hue: i32,
        entry_id: i32,
        ox: i32,
        oy: i32,
        initial_text: &str,
    ) {
        self.add_text_entry(
            c.x + ox,
            c.y + oy,
            c.width - ox,
            c.height - oy,
            hue,
            entry_id,
            initial_text,
        );
    }

    fn add_text_entry_limited_cell(
        &mut self,
        c: &GridCell,
        hue: i32,
        entry_id: i32,
        size: i32,
        initial_text: &str,
    ) {
        self.add_text_entry_limited(
            c.x,
            c.y,
            c.width,
            c.height,
            hue,
            entry_id,
            initial_text,
            size,
        );
    }

    fn add_image_tiled_button_cell(
        &mut self,
        c: &GridCell,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        kind: GumpButtonType,
        param: i32,
        item_id: i32,
        hue: i32,
        localized_tooltip: i32,
    ) {
        self.add_image_tiled_button(
            c.x,
            c.y,
            normal_id,
            pressed_id,
            button_id,
            kind,
            param,
            item_id,
            hue,
            c.width,
            c.height,
            localized_tooltip,
        );
    }

    fn add_sprite_image_cell(&mut self, c: &GridCell, gump_id: i32, sx: i32, sy: i32) {
        self.add_sprite_image(c.x, c.y, gump_id, c.width, c.height, sx, sy);
    }

    fn add_sprite_image_cell_sized(
        &mut self,
        c: &GridCell,
        gump_id: i32,
        width: i32,
        height: i32,
        sx: i32,
        sy: i32,
        ox: i32,
        oy: i32,
    ) {
        self.add_sprite_image(c.x + ox, c.y + oy, gump_id, width, height, sx, sy);
    }

    fn add_entry_label(&mut self, c: &GridCell, style: &GridEntryStyle, text: &str) {
        self.add_image_tiled_cell(c, style.entry_gump_id);
        self.add_label_cropped_cell(c, style.text_hue, text, style.text_offset_x, 0);
    }

    fn add_entry_html(&mut self, c: &GridCell, style: &GridEntryStyle, text: &str) {
        self.add_image_tiled_cell(c, style.entry_gump_id);
        self.add_html_cell_offset(
            c,
            text,
            style.text_offset_x,
            0,
            HtmlStyle::NONE,
            false,
            false,
        );
    }

    fn add_entry_header(&mut self, c: &GridCell, style: &GridEntryStyle) {
        self.add_image_tiled_cell(c, style.header_gump_id);
    }

    fn add_entry_button(
        &mut self,
        c: &GridCell,
        style: &GridEntryStyle,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        button_width: i32,
        button_height: i32,
    ) {
        self.add_image_tiled_cell(c, style.header_gump_id);
        self.add_button_cell(
            c,
            normal_id,
            pressed_id,
            button_id,
            GumpButtonType::Reply,
            0,
            (c.width - button_width) / 2,
            (c.height - button_height) / 2,
        );
    }

    fn add_entry_arrow_left(&mut self, c: &GridCell, style: &GridEntryStyle, button_id: i32) {
        self.add_entry_button(
            c,
            style,
            GridEntryStyle::ARROW_LEFT_ID1,
            GridEntryStyle::ARROW_LEFT_ID2,
            button_id,
            GridEntryStyle::ARROW_WIDTH,
            GridEntryStyle::ARROW_HEIGHT,
        );
    }

    fn add_entry_arrow_right(&mut self, c: &GridCell, style: &GridEntryStyle, button_id: i32) {
        self.add_entry_button(
            c,
            style,
            GridEntryStyle::ARROW_RIGHT_ID1,
            GridEntryStyle::ARROW_RIGHT_ID2,
            button_id,
            GridEntryStyle::ARROW_WIDTH,
            GridEntryStyle::ARROW_HEIGHT,
        );
    }

    fn add_entry_text(
        &mut self,
        c: &GridCell,
        style: &GridEntryStyle,
        entry_id: i32,
        initial_text: &str,
    ) {
        self.add_image_tiled_cell(c, style.entry_gump_id);
        self.add_text_entry_cell_offset(
            c,
            style.text_hue,
            entry_id,
            style.text_offset_x,
            0,
            initial_text,
        );
    }

    fn add_entry_blank(&mut self, c: &GridCell, style: &GridEntryStyle) {
        self.add_image_tiled_cell(c, style.back_gump_id + 4);
    }

    fn add_grid_background(&mut self, width: i32, height: i32, style: &GridEntryStyle) {
        self.add_background(0, 0, width, height, style.back_gump_id);
        self.add_image_tiled(
            style.border_size,
            style.border_size,
            width - style.border_size * 2,
            height - style.border_size * 2,
            style.offset_gump_id,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_absolute_star_percent() {
        assert_eq!(GridSizeSpec::parse("100"), GridSizeSpec::absolute(100));
        assert_eq!(GridSizeSpec::parse("*"), GridSizeSpec::star());
        assert_eq!(GridSizeSpec::parse("10*"), GridSizeSpec::percent(10));
        assert_eq!(GridSizeSpec::parse("abc"), GridSizeSpec::star());
    }

    #[test]
    fn parse_all() {
        let mut specs = [GridSizeSpec::star(); 4];
        let count = GridSizeSpec::parse_all("10* * 100 20*", &mut specs);
        assert_eq!(count, 4);
        assert_eq!(
            specs,
            [
                GridSizeSpec::percent(10),
                GridSizeSpec::star(),
                GridSizeSpec::absolute(100),
                GridSizeSpec::percent(20)
            ]
        );
    }

    #[test]
    fn uniform_tracks() {
        let (mut p, mut s) = ([0; 4], [0; 4]);
        compute_uniform_track_sizes(4, 400, 0, &mut p, &mut s);
        assert_eq!(p, [0, 100, 200, 300]);
        assert_eq!(s, [100; 4]);
        let (mut p, mut s) = ([0; 2], [0; 2]);
        compute_uniform_track_sizes(2, 200, 50, &mut p, &mut s);
        assert_eq!((p, s), ([50, 150], [100, 100]));
    }

    #[test]
    fn mixed_tracks() {
        let (mut p, mut s) = ([0; 3], [0; 3]);
        let count = compute_from_spec("10* * 100", 1000, 0, &mut p, &mut s);
        assert_eq!(count, 3);
        assert_eq!((p[0], s[0]), (0, 100));
        assert_eq!((p[2], s[2]), (900, 100));
        assert_eq!((p[1], s[1]), (100, 800));
    }

    fn layout(total: i32, page: i32) -> ListViewLayout {
        let (mut p, mut w) = ([0; 3], [0; 3]);
        ListViewLayout::create(0, 0, 300, 210, total, page, 20, 10, "* * *", &mut p, &mut w)
    }

    #[test]
    fn list_pagination() {
        let l = layout(100, 0);
        assert_eq!(
            (l.total_items, l.items_per_page, l.current_page),
            (100, 10, 0)
        );
        assert_eq!((l.total_pages, l.start_index, l.visible_count), (10, 0, 10));
        assert!(!l.can_go_back && l.can_go_next);

        let l = layout(100, 5);
        assert_eq!(
            (l.current_page, l.start_index, l.visible_count),
            (5, 50, 10)
        );
        assert!(l.can_go_back && l.can_go_next);

        let l = layout(95, 9);
        assert_eq!((l.current_page, l.start_index, l.visible_count), (9, 90, 5));
        assert!(l.can_go_back && !l.can_go_next);

        let l = layout(0, 0);
        assert_eq!(
            (l.total_items, l.total_pages, l.start_index, l.visible_count),
            (0, 1, 0, 0)
        );
        assert!(!l.can_go_back && !l.can_go_next);
    }

    #[test]
    fn list_cells() {
        let (mut p, mut w) = ([0; 3], [0; 3]);
        let l = ListViewLayout::create(
            10,
            20,
            300,
            200,
            50,
            0,
            20,
            30,
            "100 100 100",
            &mut p,
            &mut w,
        );
        assert_eq!(
            l.get_header_cell(1, &p, &w),
            GridCell::new(110, 20, 100, 30)
        );
        let (mut p, mut w) = ([0; 3], [0; 3]);
        let l =
            ListViewLayout::create(0, 0, 300, 200, 50, 0, 20, 30, "100 100 100", &mut p, &mut w);
        assert_eq!(
            l.get_row_cell(2, 1, &p, &w),
            GridCell::new(100, 70, 100, 20)
        );
    }

    #[test]
    fn spawner_column_spec() {
        let (mut p, mut w) = ([0; 12], [0; 12]);
        let l = ListViewLayout::create(
            0,
            80,
            1000,
            620,
            100,
            0,
            45,
            30,
            "6* 8* 18* 6* 12* 6* 5* 5* 8* 8* 9* *",
            &mut p,
            &mut w,
        );
        assert_eq!(l.column_count, 12);
        assert_eq!((p[0], w[0]), (0, 60));
        assert_eq!((p[1], w[1]), (60, 80));
    }

    #[test]
    fn cell_math() {
        let c = GridCell::new(10, 20, 100, 50);
        assert_eq!(c.inset(5), GridCell::new(15, 25, 90, 40));
        assert_eq!(
            (c.right(), c.bottom(), c.center_x(), c.center_y()),
            (110, 70, 60, 45)
        );
        let style = GridEntryStyle::DEFAULT;
        assert_eq!(style.total_height(3), 20 + 2 + 60 + 2);
    }
}
