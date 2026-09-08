use std::io::IsTerminal;

#[derive(Clone, Copy, PartialEq)]
pub enum Depth {
    None,
    Ansi16,
    TrueColor,
}

pub fn detect() -> Depth {
    if std::env::var_os("NO_COLOR").is_some() || !std::io::stdout().is_terminal() {
        return Depth::None;
    }
    match std::env::var("COLORTERM") {
        Ok(v) if v.contains("truecolor") || v.contains("24bit") => Depth::TrueColor,
        _ => Depth::Ansi16,
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub rgb: (u8, u8, u8),
    pub ansi_fg: &'static str,
}

pub const TEXT: Color = Color {
    rgb: (0xed, 0xeb, 0xe7),
    ansi_fg: "37",
};
pub const MUTED: Color = Color {
    rgb: (0x9b, 0x95, 0x8c),
    ansi_fg: "90",
};
pub const FAINT: Color = Color {
    rgb: (0x6e, 0x6a, 0x63),
    ansi_fg: "90",
};
pub const TEAL: Color = Color {
    rgb: (0x40, 0xd0, 0xd5),
    ansi_fg: "36",
};
pub const AMBER: Color = Color {
    rgb: (0xf5, 0xb7, 0x00),
    ansi_fg: "33",
};
pub const ORANGE: Color = Color {
    rgb: (0xff, 0x70, 0x38),
    ansi_fg: "31",
};

const BADGE_BG: (u8, u8, u8) = (0x1f, 0x1e, 0x1c);
const BADGE_BG_256: &str = "236";

pub struct Painter {
    depth: Depth,
}

impl Painter {
    pub fn new() -> Self {
        Painter { depth: detect() }
    }

    pub fn fg(&self, text: &str, c: Color) -> String {
        match self.depth {
            Depth::None => text.to_string(),
            Depth::Ansi16 => format!("\x1b[{}m{text}\x1b[0m", c.ansi_fg),
            Depth::TrueColor => {
                let (r, g, b) = c.rgb;
                format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
            }
        }
    }

    pub fn bold(&self, text: &str) -> String {
        if self.depth == Depth::None {
            text.to_string()
        } else {
            format!("\x1b[1m{text}\x1b[0m")
        }
    }

    pub fn badge(&self, text: &str, fg: Color) -> String {
        match self.depth {
            Depth::None => format!("[{text}]"),
            Depth::Ansi16 => {
                format!(
                    "\x1b[48;5;{BADGE_BG_256}m\x1b[{}m {text} \x1b[0m",
                    fg.ansi_fg
                )
            }
            Depth::TrueColor => {
                let (br, bg, bb) = BADGE_BG;
                let (fr, fgc, fb) = fg.rgb;
                format!("\x1b[48;2;{br};{bg};{bb}m\x1b[38;2;{fr};{fgc};{fb}m {text} \x1b[0m")
            }
        }
    }
}
