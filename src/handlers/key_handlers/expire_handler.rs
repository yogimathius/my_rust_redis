use crate::{
    log,
    models::value::Value,
    server::Server,
    utilities::{should_set_expiry, unpack_bulk_str, unpack_integer},
};
use std::time::Instant;

pub fn expire_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    let expiration_time = unpack_integer(args.get(1).unwrap().clone()).unwrap();

    let option = match args.get(2) {
        Some(value) => unpack_bulk_str(value.clone()).unwrap_or_default(),
        None => String::new(),
    };

    log!("option {:?}", option);
    let mut cache = server.cache.lock().unwrap();

    match cache.get_mut(&key) {
        Some(item) => {
            log!("value {:?}", item);
            if should_set_expiry(item, expiration_time, option) {
                log!("setting expiration");
                let now = Instant::now();
                let new_expiration =
                    now + std::time::Duration::from_secs(expiration_time.try_into().unwrap());
                let new_expiration_secs =
                    new_expiration.duration_since(Instant::now()).as_secs() as i64;

                item.expiration = Some(new_expiration_secs);
                Some(Value::Integer(1))
            } else {
                Some(Value::Integer(0))
            }
        }
        None => Some(Value::Integer(0)),
    }
}
