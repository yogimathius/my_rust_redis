/**
NX -- Set expiry only when the key has no expiry
XX -- Set expiry only when the key has an existing expiry
GT -- Set expiry only when the new expiry is greater than current one
LT -- Set expiry only when the new expiry is less than current one
 */
// helper function
use crate::server::RedisItem;

pub fn should_set_expiry(item: &RedisItem, expiration: i64, option: Option<String>) -> bool {
    println!("item {:?}", item);
    println!("expiration {:?}", expiration);
    println!("option {:?}", option);

    match option.as_deref() {
        Some("NX") => {
            return item.expiration.is_none();
        }
        Some("XX") => {
            return item.expiration.is_some();
        }
        Some("GT") => {
            return item.expiration.is_some() && item.expiration.unwrap() < expiration;
        }
        Some("LT") => {
            return item.expiration.is_some() && item.expiration.unwrap() > expiration;
        }
        None => {
            println!("no option");
            return true;
        }
        _ => {
            return false;
        }
    }
}
