use crate::{models::value::Value, server::Server, utilities::unpack_bulk_str};

pub fn del_handler(server: &mut Server, _: String, args: Vec<Value>) -> Option<Value> {
    let keys = args
        .iter()
        .map(|arg| unpack_bulk_str(arg.clone()).unwrap())
        .collect::<Vec<String>>();

    let mut cache = server.cache.lock().unwrap();

    let mut count = 0;

    for key in keys {
        if cache.remove(&key).is_some() {
            count += 1;
        }
    }

    Some(Value::Integer(count))
}
