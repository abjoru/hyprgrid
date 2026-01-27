use crate::theme::Theme;

pub fn generate_css(theme: &Theme) -> String {
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
    padding: 10px;
    min-width: 180px;
    min-height: 60px;
    transition: all 150ms ease-in-out;
}}

.grid-cell.selected {{
    border: 2px solid {border};
    box-shadow: 0 0 8px alpha({border}, 0.5);
}}

.cell-name {{
    color: {fg};
    font-weight: bold;
    font-size: 14px;
}}

.cell-desc {{
    color: {fg_dim};
    font-size: 11px;
}}
"#,
        border = theme.border,
        fg = theme.fg,
        fg_dim = theme.fg_dim,
    );

    // Generate accent color classes
    let border = &theme.border;
    for (i, color) in theme.accents.iter().enumerate() {
        css.push_str(&format!(
            r#"
.cell-accent-{i} {{
    background-color: {color};
    border: 1px solid {color};
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
