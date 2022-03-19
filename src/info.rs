use std::net::SocketAddr;

use colored::*;

use crate::{USERS, FluxUser, OFFERS};

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
                info!("{}{}{} {}", "[".bright_black(), USERS.get(i).expect("Can get username in user_info method").name.clone().color(color), "]".bright_black(), details);
                ()
            },
            None => info!("{}{}{} {}", "[".bright_black(), addr.to_string().color(color), "]".bright_black(), details),
        }
    }
}

/**
 * Custom error function to log client debug info.
 */
pub fn user_err(addr: SocketAddr, details: String) {
    unsafe {
        let index = USERS.iter().position(|user| user.addr == addr);

        match index {
            Some(i) => {
                info!("{}{}{} {}{}{} {}", "[".bright_black(), "ERR".bright_red(), "]".bright_black(), "[".bright_black(), USERS.get(i).expect("Can get username in user_info method").name.clone(), "]".bright_black(), details);
                ()
            },
            None => info!("{}{}{} {}{}{} {}", "[".bright_black(), "ERR".bright_red(), "]".bright_black(), "[".bright_black(), addr.to_string(), "]".bright_black(), details),
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

/**
 * Get a user based on their id.
 */
pub fn get_user_id<'a>(id: String) -> Option<&'a FluxUser> {
    if user_id_exists(id.clone()) == false {
        return None;
    }

    unsafe {
        let index = USERS.iter().position(|user| user.id == id);

        match index {
            Some(i) => {
                return Some(USERS.get(i).unwrap());
            },
            None => panic!("Tried to get user who doesn't exist"),
        }
    }
}

/**
 * Check if a user is logged in.
 */
pub fn user_exists(addr: SocketAddr) -> bool {
    unsafe {
        let result = USERS.iter().any(|user| user.addr == addr);
        if result == false {
            user_info(
                addr,
                String::from("Unauthorized (Not logged in)"),
                Color::Red
            );
        }
        return result;
    }
}

/**
 * Check if a user is logged in.
 */
pub fn user_id_exists(id: String) -> bool {
    unsafe {
        return USERS.iter().any(|user| user.id == id);
    }
}

/**
 * Dispose of an offer by id.
 */
pub fn dispose_offer(id: String) {
    unsafe {
        let index = OFFERS.iter().position(|offer| offer.id == id);

        match index {
            Some(i) => {
                OFFERS.remove(i);
            },
            None => panic!("Tried to dispose of offer which doesn't exist"),
        }
    }
}