use gtk4::gdk::Key;

pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Launch,
    Close,
    None,
}

pub fn parse_key(key: Key) -> Action {
    match key {
        Key::h | Key::Left => Action::MoveLeft,
        Key::l | Key::Right => Action::MoveRight,
        Key::k | Key::Up => Action::MoveUp,
        Key::j | Key::Down => Action::MoveDown,
        Key::Return | Key::KP_Enter => Action::Launch,
        Key::Escape | Key::q => Action::Close,
        _ => Action::None,
    }
}
