use logtest::Logger;

#[test]
fn smoke() {
    let mut logger = Logger::start();
    log::info!("hello");
    log::info!("world");
    assert_eq!(logger.len(), 2);
    assert_eq!(logger.pop_front().unwrap().args(), "hello");
    assert_eq!(logger.pop_front().unwrap().args(), "world");
    assert_eq!(logger.len(), 0);

    kv_log_macro::info!("hello", { color: "blue" });
    kv_log_macro::info!("world", { name: "chashu" });
    assert_eq!(logger.len(), 2);

    let msg = logger.pop_front().unwrap();
    assert_eq!(msg.args(), "hello");
    assert_eq!(
        msg.key_values(),
        vec![("color".to_owned(), "\"blue\"".to_owned())]
    );

    let msg = logger.pop_front().unwrap();
    assert_eq!(msg.args(), "world");
    assert_eq!(
        msg.key_values(),
        vec![("name".to_owned(), "\"chashu\"".to_owned())]
    );

    assert_eq!(logger.len(), 0);
}
