use crate::theme::{Theme, desaturate};

pub fn generate_css(theme: &Theme) -> String {
    let dim = theme.dim_strength;
    let fg_dimmed = desaturate(&theme.fg, dim);
    let fg_dim_dimmed = desaturate(&theme.fg_dim, dim);

    let mut css = format!(
        r#"
window {{
    background-color: rgba(0, 0, 0, 0);
}}

frame, .grid-container {{
    background-color: rgba(0, 0, 0, 0);
    border: none;
    padding: 12px;
}}

frame > border {{
    background-color: rgba(0, 0, 0, 0);
    border: none;
}}

.grid-cell {{
    border-radius: 6px;
    border: 1px solid {fg_dim};
    padding: 10px;
    min-width: 180px;
    min-height: 60px;
    transition: all 150ms ease-in-out;
}}

.grid-cell.selected {{
    border: 2px solid {border};
    box-shadow: 0 0 8px alpha({border}, 0.5);
}}

.grid-cell .cell-name {{
    color: {fg_dimmed};
    font-weight: bold;
    font-size: 14px;
}}

.grid-cell.selected .cell-name {{
    color: {fg};
}}

.grid-cell .cell-desc {{
    color: {fg_dim_dimmed};
    font-size: 11px;
}}

.grid-cell.selected .cell-desc {{
    color: {fg_dim};
}}

.cell-icon {{
    margin-right: 4px;
}}
"#,
        border = theme.border,
        fg = theme.fg,
        fg_dim = theme.fg_dim,
        fg_dimmed = fg_dimmed,
        fg_dim_dimmed = fg_dim_dimmed,
    );

    // Generate accent color classes with dimmed variants for unselected
    let border = &theme.border;
    let fg_dim = &theme.fg_dim;
    for (i, color) in theme.accents.iter().enumerate() {
        let dimmed = desaturate(color, dim);
        css.push_str(&format!(
            r#"
.cell-accent-{i} {{
    background-color: {dimmed};
    border: 1px solid {fg_dim};
}}

.cell-accent-{i}.selected {{
    background-color: {color};
    border: 2px solid {border};
}}
"#
        ));
    }

    css
}

pub fn accent_class(index: usize, num_accents: usize) -> String {
    format!("cell-accent-{}", index % num_accents)
}
