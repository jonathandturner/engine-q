use nu_ansi_term::{Color, Style};
use nu_protocol::Config;
use nu_table::{Alignment, TextStyle};
use serde::Deserialize;
use std::collections::HashMap;

//TODO: should this be implemented again?
// pub fn number(number: impl Into<Number>) -> Primitive {
//     let number = number.into();

//     match number {
//         Number::BigInt(int) => Primitive::BigInt(int),
//         Number::Int(int) => Primitive::Int(int),
//         Number::Decimal(decimal) => Primitive::Decimal(decimal),
//     }
// }

#[derive(Deserialize, PartialEq, Debug)]
struct NuStyle {
    fg: Option<String>,
    bg: Option<String>,
    attr: Option<String>,
}

fn parse_nustyle(nu_style: NuStyle) -> Style {
    // get the nu_ansi_term::Color foreground color
    let fg_color = match nu_style.fg {
        Some(fg) => color_from_hex(&fg).expect("error with foreground color"),
        _ => None,
    };
    // get the nu_ansi_term::Color background color
    let bg_color = match nu_style.bg {
        Some(bg) => color_from_hex(&bg).expect("error with background color"),
        _ => None,
    };
    // get the attributes
    let color_attr = match nu_style.attr {
        Some(attr) => attr,
        _ => "".to_string(),
    };

    // setup the attributes available in nu_ansi_term::Style
    let mut bold = false;
    let mut dimmed = false;
    let mut italic = false;
    let mut underline = false;
    let mut blink = false;
    let mut reverse = false;
    let mut hidden = false;
    let mut strikethrough = false;

    // since we can combine styles like bold-italic, iterate through the chars
    // and set the bools for later use in the nu_ansi_term::Style application
    for ch in color_attr.to_lowercase().chars() {
        match ch {
            'l' => blink = true,
            'b' => bold = true,
            'd' => dimmed = true,
            'h' => hidden = true,
            'i' => italic = true,
            'r' => reverse = true,
            's' => strikethrough = true,
            'u' => underline = true,
            'n' => (),
            _ => (),
        }
    }

    // here's where we build the nu_ansi_term::Style
    Style {
        foreground: fg_color,
        background: bg_color,
        is_blink: blink,
        is_bold: bold,
        is_dimmed: dimmed,
        is_hidden: hidden,
        is_italic: italic,
        is_reverse: reverse,
        is_strikethrough: strikethrough,
        is_underline: underline,
    }
}

fn color_string_to_nustyle(color_string: String) -> Style {
    // eprintln!("color_string: {}", &color_string);
    if color_string.chars().count() < 1 {
        Style::default()
    } else {
        let nu_style = match nu_json::from_str::<NuStyle>(&color_string) {
            Ok(s) => s,
            Err(_) => NuStyle {
                fg: None,
                bg: None,
                attr: None,
            },
        };

        parse_nustyle(nu_style)
    }
}

fn color_from_hex(hex_color: &str) -> std::result::Result<Option<Color>, std::num::ParseIntError> {
    // right now we only allow hex colors with hashtag and 6 characters
    let trimmed = hex_color.trim_matches('#');
    if trimmed.len() != 6 {
        Ok(None)
    } else {
        // make a nu_ansi_term::Color::Rgb color by converting hex to decimal
        Ok(Some(Color::Rgb(
            u8::from_str_radix(&trimmed[..2], 16)?,
            u8::from_str_radix(&trimmed[2..4], 16)?,
            u8::from_str_radix(&trimmed[4..6], 16)?,
        )))
    }
}

pub fn lookup_ansi_color_style(s: String) -> Style {
    if s.starts_with('#') {
        match color_from_hex(&s) {
            Ok(c) => match c {
                Some(c) => c.normal(),
                None => Style::default(),
            },
            Err(_) => Style::default(),
        }
    } else if s.starts_with('{') {
        color_string_to_nustyle(s)
    } else {
        match s.as_str() {
            "g" | "green" => Color::Green.normal(),
            "gb" | "green_bold" => Color::Green.bold(),
            "gu" | "green_underline" => Color::Green.underline(),
            "gi" | "green_italic" => Color::Green.italic(),
            "gd" | "green_dimmed" => Color::Green.dimmed(),
            "gr" | "green_reverse" => Color::Green.reverse(),
            "gbl" | "green_blink" => Color::Green.blink(),
            "gst" | "green_strike" => Color::Green.strikethrough(),
            "r" | "red" => Color::Red.normal(),
            "rb" | "red_bold" => Color::Red.bold(),
            "ru" | "red_underline" => Color::Red.underline(),
            "ri" | "red_italic" => Color::Red.italic(),
            "rd" | "red_dimmed" => Color::Red.dimmed(),
            "rr" | "red_reverse" => Color::Red.reverse(),
            "rbl" | "red_blink" => Color::Red.blink(),
            "rst" | "red_strike" => Color::Red.strikethrough(),
            "u" | "blue" => Color::Blue.normal(),
            "ub" | "blue_bold" => Color::Blue.bold(),
            "uu" | "blue_underline" => Color::Blue.underline(),
            "ui" | "blue_italic" => Color::Blue.italic(),
            "ud" | "blue_dimmed" => Color::Blue.dimmed(),
            "ur" | "blue_reverse" => Color::Blue.reverse(),
            "ubl" | "blue_blink" => Color::Blue.blink(),
            "ust" | "blue_strike" => Color::Blue.strikethrough(),
            "b" | "black" => Color::Black.normal(),
            "bb" | "black_bold" => Color::Black.bold(),
            "bu" | "black_underline" => Color::Black.underline(),
            "bi" | "black_italic" => Color::Black.italic(),
            "bd" | "black_dimmed" => Color::Black.dimmed(),
            "br" | "black_reverse" => Color::Black.reverse(),
            "bbl" | "black_blink" => Color::Black.blink(),
            "bst" | "black_strike" => Color::Black.strikethrough(),
            "y" | "yellow" => Color::Yellow.normal(),
            "yb" | "yellow_bold" => Color::Yellow.bold(),
            "yu" | "yellow_underline" => Color::Yellow.underline(),
            "yi" | "yellow_italic" => Color::Yellow.italic(),
            "yd" | "yellow_dimmed" => Color::Yellow.dimmed(),
            "yr" | "yellow_reverse" => Color::Yellow.reverse(),
            "ybl" | "yellow_blink" => Color::Yellow.blink(),
            "yst" | "yellow_strike" => Color::Yellow.strikethrough(),
            "p" | "purple" => Color::Purple.normal(),
            "pb" | "purple_bold" => Color::Purple.bold(),
            "pu" | "purple_underline" => Color::Purple.underline(),
            "pi" | "purple_italic" => Color::Purple.italic(),
            "pd" | "purple_dimmed" => Color::Purple.dimmed(),
            "pr" | "purple_reverse" => Color::Purple.reverse(),
            "pbl" | "purple_blink" => Color::Purple.blink(),
            "pst" | "purple_strike" => Color::Purple.strikethrough(),
            "c" | "cyan" => Color::Cyan.normal(),
            "cb" | "cyan_bold" => Color::Cyan.bold(),
            "cu" | "cyan_underline" => Color::Cyan.underline(),
            "ci" | "cyan_italic" => Color::Cyan.italic(),
            "cd" | "cyan_dimmed" => Color::Cyan.dimmed(),
            "cr" | "cyan_reverse" => Color::Cyan.reverse(),
            "cbl" | "cyan_blink" => Color::Cyan.blink(),
            "cst" | "cyan_strike" => Color::Cyan.strikethrough(),
            "w" | "white" => Color::White.normal(),
            "wb" | "white_bold" => Color::White.bold(),
            "wu" | "white_underline" => Color::White.underline(),
            "wi" | "white_italic" => Color::White.italic(),
            "wd" | "white_dimmed" => Color::White.dimmed(),
            "wr" | "white_reverse" => Color::White.reverse(),
            "wbl" | "white_blink" => Color::White.blink(),
            "wst" | "white_strike" => Color::White.strikethrough(),
            _ => Color::White.normal(),
        }
    }
}

// TODO: i'm not sure how this ever worked but leaving it in case it's used elsewhere but not implemented yet
// pub fn string_to_lookup_value(str_prim: &str) -> String {
//     match str_prim {
//         "primitive_int" => "Primitive::Int".to_string(),
//         "primitive_decimal" => "Primitive::Decimal".to_string(),
//         "primitive_filesize" => "Primitive::Filesize".to_string(),
//         "primitive_string" => "Primitive::String".to_string(),
//         "primitive_line" => "Primitive::Line".to_string(),
//         "primitive_columnpath" => "Primitive::ColumnPath".to_string(),
//         "primitive_pattern" => "Primitive::GlobPattern".to_string(),
//         "primitive_boolean" => "Primitive::Boolean".to_string(),
//         "primitive_date" => "Primitive::Date".to_string(),
//         "primitive_duration" => "Primitive::Duration".to_string(),
//         "primitive_range" => "Primitive::Range".to_string(),
//         "primitive_path" => "Primitive::FilePath".to_string(),
//         "primitive_binary" => "Primitive::Binary".to_string(),
//         "separator_color" => "separator_color".to_string(),
//         "header_align" => "header_align".to_string(),
//         "header_color" => "header_color".to_string(),
//         "header_style" => "header_style".to_string(),
//         "index_color" => "index_color".to_string(),
//         "leading_trailing_space_bg" => "leading_trailing_space_bg".to_string(),
//         _ => "Primitive::Nothing".to_string(),
//     }
// }

fn update_hashmap(key: &str, val: &str, hm: &mut HashMap<String, Style>) {
    // eprintln!("key: {}, val: {}", &key, &val);
    let color = lookup_ansi_color_style(val.to_string());
    if let Some(v) = hm.get_mut(key) {
        *v = color;
    } else {
        hm.insert(key.to_string(), color);
    }
}

pub fn get_color_config(config: &Config) -> HashMap<String, Style> {
    let config = config;

    // create the hashmap
    let mut hm: HashMap<String, Style> = HashMap::new();
    // set some defaults
    // hm.insert("primitive_int".to_string(), Color::White.normal());
    // hm.insert("primitive_decimal".to_string(), Color::White.normal());
    // hm.insert("primitive_filesize".to_string(), Color::White.normal());
    // hm.insert("primitive_string".to_string(), Color::White.normal());
    // hm.insert("primitive_line".to_string(), Color::White.normal());
    // hm.insert("primitive_columnpath".to_string(), Color::White.normal());
    // hm.insert("primitive_pattern".to_string(), Color::White.normal());
    // hm.insert("primitive_boolean".to_string(), Color::White.normal());
    // hm.insert("primitive_date".to_string(), Color::White.normal());
    // hm.insert("primitive_duration".to_string(), Color::White.normal());
    // hm.insert("primitive_range".to_string(), Color::White.normal());
    // hm.insert("primitive_path".to_string(), Color::White.normal());
    // hm.insert("primitive_binary".to_string(), Color::White.normal());
    // hm.insert("separator_color".to_string(), Color::White.normal());
    // hm.insert("header_align".to_string(), Color::Green.bold());
    // hm.insert("header_color".to_string(), Color::Green.bold());
    // hm.insert("header_style".to_string(), Style::default());
    // hm.insert("index_color".to_string(), Color::Green.bold());
    hm.insert(
        "leading_trailing_space_bg".to_string(),
        Style::default().on(Color::Rgb(128, 128, 128)),
    );

    hm.insert("header".to_string(), Color::Green.bold());
    hm.insert("empty".to_string(), Color::Blue.normal());

    hm.insert("bool".to_string(), Color::White.normal());
    hm.insert("int".to_string(), Color::White.normal());
    hm.insert("filesize".to_string(), Color::White.normal());
    hm.insert("duration".to_string(), Color::White.normal());
    hm.insert("date".to_string(), Color::White.normal());
    hm.insert("range".to_string(), Color::White.normal());
    hm.insert("float".to_string(), Color::White.normal());
    hm.insert("string".to_string(), Color::White.normal());
    hm.insert("nothing".to_string(), Color::White.normal());
    hm.insert("binary".to_string(), Color::White.normal());
    hm.insert("cellpath".to_string(), Color::White.normal());
    hm.insert("row_index".to_string(), Color::Green.bold());
    hm.insert("record".to_string(), Color::White.normal());
    hm.insert("list".to_string(), Color::White.normal());
    hm.insert("block".to_string(), Color::White.normal());

    for (key, value) in &config.color_config {
        update_hashmap(key, value, &mut hm);

        // eprintln!(
        //     "config: {}:{}\t\t\thashmap: {}:{:?}",
        //     &key, &value, &key, &hm[key]
        // );
    }

    hm
}

// This function will assign a text style to a primitive, or really any string that's
// in the hashmap. The hashmap actually contains the style to be applied.
pub fn style_primitive(primitive: &str, color_hm: &HashMap<String, Style>) -> TextStyle {
    match primitive {
        "bool" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        "int" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Right, *s),
                None => TextStyle::basic_right(),
            }
        }

        "filesize" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Right, *s),
                None => TextStyle::basic_right(),
            }
        }

        "duration" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        "date" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        "range" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        "float" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Right, *s),
                None => TextStyle::basic_right(),
            }
        }

        "string" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        "record" | "list" | "block" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        "nothing" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        // not sure what to do with error
        // "error" => {}
        "binary" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        "cellpath" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Left, *s),
                None => TextStyle::basic_left(),
            }
        }

        "row_index" => {
            let style = color_hm.get(primitive);
            match style {
                Some(s) => TextStyle::with_style(Alignment::Right, *s),
                None => TextStyle::new()
                    .alignment(Alignment::Right)
                    .fg(Color::Green)
                    .bold(Some(true)),
            }
        }

        // types in nushell but not in engine-q
        // "Line" => {
        //     let style = color_hm.get("Primitive::Line");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Left, *s),
        //         None => TextStyle::basic_left(),
        //     }
        // }
        // "GlobPattern" => {
        //     let style = color_hm.get("Primitive::GlobPattern");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Left, *s),
        //         None => TextStyle::basic_left(),
        //     }
        // }
        // "FilePath" => {
        //     let style = color_hm.get("Primitive::FilePath");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Left, *s),
        //         None => TextStyle::basic_left(),
        //     }
        // }
        // "BeginningOfStream" => {
        //     let style = color_hm.get("Primitive::BeginningOfStream");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Left, *s),
        //         None => TextStyle::basic_left(),
        //     }
        // }
        // "EndOfStream" => {
        //     let style = color_hm.get("Primitive::EndOfStream");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Left, *s),
        //         None => TextStyle::basic_left(),
        //     }
        // }
        // "separator_color" => {
        //     let style = color_hm.get("separator");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Left, *s),
        //         None => TextStyle::basic_left(),
        //     }
        // }
        // "header_align" => {
        //     let style = color_hm.get("header_align");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Center, *s),
        //         None => TextStyle::default_header(),
        //     }
        // }
        // "header_color" => {
        //     let style = color_hm.get("header_color");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Center, *s),
        //         None => TextStyle::default_header().bold(Some(true)),
        //     }
        // }
        // "header_style" => {
        //     let style = color_hm.get("header_style");
        //     match style {
        //         Some(s) => TextStyle::with_style(Alignment::Center, *s),
        //         None => TextStyle::default_header(),
        //     }
        // }
        _ => TextStyle::basic_left(),
    }
}

pub fn get_shape_color(shape: String, conf: &Config) -> Style {
    match shape.as_ref() {
        "flatshape_garbage" => {
            if conf.color_config.contains_key("flatshape_garbage") {
                let int_color = &conf.color_config["flatshape_garbage"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::White).on(Color::Red).bold()
            }
        }
        "flatshape_bool" => {
            if conf.color_config.contains_key("flatshape_bool") {
                let int_color = &conf.color_config["flatshape_bool"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::LightCyan)
            }
        }
        "flatshape_int" => {
            if conf.color_config.contains_key("flatshape_int") {
                let int_color = &conf.color_config["flatshape_int"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Purple).bold()
            }
        }
        "flatshape_float" => {
            if conf.color_config.contains_key("flatshape_float") {
                let int_color = &conf.color_config["flatshape_float"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Purple).bold()
            }
        }
        "flatshape_range" => {
            if conf.color_config.contains_key("flatshape_range") {
                let int_color = &conf.color_config["flatshape_range"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Yellow).bold()
            }
        }
        "flatshape_internalcall" => {
            if conf.color_config.contains_key("flatshape_internalcall") {
                let int_color = &conf.color_config["flatshape_internalcall"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Cyan).bold()
            }
        }
        "flatshape_external" => {
            if conf.color_config.contains_key("flatshape_external") {
                let int_color = &conf.color_config["flatshape_external"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Cyan)
            }
        }
        "flatshape_externalarg" => {
            if conf.color_config.contains_key("flatshape_externalarg") {
                let int_color = &conf.color_config["flatshape_externalarg"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Green).bold()
            }
        }
        "flatshape_literal" => {
            if conf.color_config.contains_key("flatshape_literal") {
                let int_color = &conf.color_config["flatshape_literal"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Blue)
            }
        }
        "flatshape_operator" => {
            if conf.color_config.contains_key("flatshape_operator") {
                let int_color = &conf.color_config["flatshape_operator"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Yellow)
            }
        }
        "flatshape_signature" => {
            if conf.color_config.contains_key("flatshape_signature") {
                let int_color = &conf.color_config["flatshape_signature"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Green).bold()
            }
        }
        "flatshape_string" => {
            if conf.color_config.contains_key("flatshape_string") {
                let int_color = &conf.color_config["flatshape_string"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Green)
            }
        }
        "flatshape_filepath" => {
            if conf.color_config.contains_key("flatshape_filepath") {
                let int_color = &conf.color_config["flatshape_filepath"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Cyan)
            }
        }
        "flatshape_globpattern" => {
            if conf.color_config.contains_key("flatshape_globpattern") {
                let int_color = &conf.color_config["flatshape_globpattern"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Cyan).bold()
            }
        }
        "flatshape_variable" => {
            if conf.color_config.contains_key("flatshape_variable") {
                let int_color = &conf.color_config["flatshape_variable"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Purple)
            }
        }
        "flatshape_flag" => {
            if conf.color_config.contains_key("flatshape_flag") {
                let int_color = &conf.color_config["flatshape_flag"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().fg(Color::Blue).bold()
            }
        }
        "flatshape_custom" => {
            if conf.color_config.contains_key("flatshape_custom") {
                let int_color = &conf.color_config["flatshape_custom"];
                lookup_ansi_color_style(int_color.to_string())
            } else {
                Style::new().bold()
            }
        }
        _ => Style::default(),
    }
}

#[test]
fn test_hm() {
    use nu_ansi_term::{Color, Style};

    let mut hm: HashMap<String, Style> = HashMap::new();
    hm.insert("primitive_int".to_string(), Color::White.normal());
    hm.insert("primitive_decimal".to_string(), Color::White.normal());
    hm.insert("primitive_filesize".to_string(), Color::White.normal());
    hm.insert("primitive_string".to_string(), Color::White.normal());
    hm.insert("primitive_line".to_string(), Color::White.normal());
    hm.insert("primitive_columnpath".to_string(), Color::White.normal());
    hm.insert("primitive_pattern".to_string(), Color::White.normal());
    hm.insert("primitive_boolean".to_string(), Color::White.normal());
    hm.insert("primitive_date".to_string(), Color::White.normal());
    hm.insert("primitive_duration".to_string(), Color::White.normal());
    hm.insert("primitive_range".to_string(), Color::White.normal());
    hm.insert("primitive_path".to_string(), Color::White.normal());
    hm.insert("primitive_binary".to_string(), Color::White.normal());
    hm.insert("separator".to_string(), Color::White.normal());
    hm.insert("header_align".to_string(), Color::Green.bold());
    hm.insert("header".to_string(), Color::Green.bold());
    hm.insert("header_style".to_string(), Style::default());
    hm.insert("row_index".to_string(), Color::Green.bold());
    hm.insert(
        "leading_trailing_space_bg".to_string(),
        Style::default().on(Color::Rgb(128, 128, 128)),
    );

    update_hashmap("primitive_int", "green", &mut hm);

    assert_eq!(hm["primitive_int"], Color::Green.normal());
}
