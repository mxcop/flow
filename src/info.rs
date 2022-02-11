use std::net::SocketAddr;

use colored::*;

use crate::USERS;

/**
 * Custom info function to log server debug info.
 */
pub fn info(title: ColoredString, details: String) {
    info!("{}{}{} {}", "[".bright_black(), title, "]".bright_black(), details);
}

/**
 * Custom info function to log client debug info.
 */
pub fn user_info(addr: SocketAddr, details: String, color: colored::Color) {
    unsafe {
        let index = USERS.iter().position(|user| user.addr == addr);

        match index {
            Some(i) => {
                info!("{}{}{} {}", "[".bright_black(), trim_str(&USERS.get(i).expect("Can get username in user_info method").name).color(color), "]".bright_black(), details);
                ()
            },
            None => info!("{}{}{} {}", "[".bright_black(), addr.to_string().color(color), "]".bright_black(), details),
        }
    }
}

/**
 * Remove the first and last characters from a string.
 */
fn trim_str(value: &String) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}