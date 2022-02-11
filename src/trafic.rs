use std::net::SocketAddr;
use colored::*;
use serde_json::Value;

use crate::{info, USERS, FluxUser};

/**
 * Handle the login message type.
 */
pub fn login(json: Value, addr: SocketAddr) {
    if json["name"] != Value::Null {
        unsafe {
            // Check if the user isn't already logged in.
            if USERS.iter().all(|user| user.addr != addr) {
                info::info(
                    addr.to_string().white(),
                    format!("@login {}", json["name"]),
                );
                USERS.push(FluxUser { name: json["name"].to_string(), addr });
            } else {
                info::user_info(
                    addr,
                    String::from("Denied (User cannot login twice)"),
                    Color::Red
                );
            }
        }

    } else {
        info::user_info(
            addr,
            String::from("Invalid (Login missing name)"),
            Color::Red
        );
    }
}