use crate::{
    log,
    models::value::Value,
    server::Server,
    utilities::{should_set_expiry, unpack_bulk_str, unpack_integer},
};
use std::time::Instant;

pub fn expire_handler(server: &mut Server, _key: String, args: Vec<Value>) -> Option<Value> {
    log!("args {:?}", args);
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    let expiration_time = unpack_integer(args.get(1).unwrap().clone()).unwrap();

    let option = match args.get(2) {
        Some(value) => unpack_bulk_str(value.clone()),
        None => unpack_bulk_str(Value::BulkString("".to_string())),
    };

    log!("option {:?}", option);
    let mut cache = server.cache.lock().unwrap();

    match cache.get(&key) {
        Some(value) => {
            log!("value {:?}", value);
            if should_set_expiry(value, expiration_time, option.unwrap()) {
                log!("setting expiration");
                let now = Instant::now();
                let new_expiration =
                    now + std::time::Duration::from_secs(expiration_time.try_into().unwrap());
                let new_expiration_secs =
                    new_expiration.duration_since(Instant::now()).as_secs() as i64;

                let item = cache.get_mut(&key).unwrap();
                item.expiration = Some(new_expiration_secs);
                return Some(Value::Integer(1));
            } else {
                return Some(Value::Integer(0));
            }
        }
        None => return Some(Value::Integer(0)),
    }
}
