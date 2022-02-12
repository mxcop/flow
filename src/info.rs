use std::net::SocketAddr;

use colored::*;

use crate::{USERS, FluxUser, util::trim_ends};

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
                info!("{}{}{} {}", "[".bright_black(), trim_ends(USERS.get(i).expect("Can get username in user_info method").name.clone()).color(color), "]".bright_black(), details);
                ()
            },
            None => info!("{}{}{} {}", "[".bright_black(), addr.to_string().color(color), "]".bright_black(), details),
        }
    }
}

/**
 * Get a user based on their socket address.
 */
pub fn get_user<'a>(addr: SocketAddr) -> &'a FluxUser {
    unsafe {
        let index = USERS.iter().position(|user| user.addr == addr);

        match index {
            Some(i) => {
                USERS.get(i).unwrap()
            },
            None => panic!("Tried to get user who doesn't exist"),
        }
    }
}