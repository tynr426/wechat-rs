use std::collections::HashMap;

use wechat::WeChatClient;
use wechat::session::RedisStorage;

const APPID: &'static str = "wx46cd147e6ef03bae";
const SECRET: &'static str = "";
const OPENID: &'static str = "ozNTc58kA9po6oev1h0fBvNyCwnY";
const REDIS_URI: &'static str = "redis://127.0.0.1/";


#[test]
fn test_user_get() {
    let session = RedisStorage::from_url(REDIS_URI);
    let client = WeChatClient::new(APPID, SECRET, session);

    let res = client.user.get(OPENID);
    println!("{:?}",res);
    //assert!(res.is_ok());

    let user = res.unwrap();
    println!("{:?}",user);
    //assert_eq!(OPENID, &user.openid);
}

// #[test]
// fn test_user_get_with_lang() {
//     let session = RedisStorage::from_url(REDIS_URI);
//     let client = WeChatClient::new(APPID, SECRET, session);

//     let res = client.user.get_with_lang(OPENID, "zh_CN");
//     assert!(res.is_ok());

//     let user = res.unwrap();
//     assert_eq!(OPENID, &user.openid);
// }

// #[test]
// fn test_user_update_remark() {
//     let session = RedisStorage::from_url(REDIS_URI);
//     let client = WeChatClient::new(APPID, SECRET, session);

//     let res = client.user.update_remark(OPENID, "test user");
//     assert!(res.is_ok());
// }

// #[test]
// fn test_user_get_followers_with_no_next_openid() {
//     let session = RedisStorage::from_url(REDIS_URI);
//     let client = WeChatClient::new(APPID, SECRET, session);

//     let res = client.user.get_followers(None);
//     assert!(res.is_ok());
// }

// #[test]
// fn test_user_get_followers_with_next_openid() {
//     let session = RedisStorage::from_url(REDIS_URI);
//     let client = WeChatClient::new(APPID, SECRET, session);

//     let res = client.user.get_followers(Some(OPENID));
//     assert!(res.is_ok());
// }

// #[test]
// fn test_user_get_group_id() {
//     let session = RedisStorage::from_url(REDIS_URI);
//     let client = WeChatClient::new(APPID, SECRET, session);

//     let res = client.user.get_group_id(OPENID);
//     assert!(res.is_ok());
// }

// #[test]
// fn test_user_get_batch() {
//     let session = RedisStorage::from_url(REDIS_URI);
//     let client = WeChatClient::new(APPID, SECRET, session);

//     let mut user_list = vec![];
//     let mut openid1 = HashMap::new();
//     openid1.insert("openid".to_owned(), OPENID.to_owned());
//     openid1.insert("lang".to_owned(), "zh-CN".to_owned());
//     user_list.push(openid1);
//     let res = client.user.get_batch(&user_list);
//     assert!(res.is_ok());

//     let users = res.unwrap();
//     assert!(users.len() > 0);
// }

// #[test]
// fn test_user_get_batch_with_lang() {
//     let session = RedisStorage::from_url(REDIS_URI);
//     let client = WeChatClient::new(APPID, SECRET, session);

//     let user_list = vec![OPENID.to_owned()];
//     let res = client.user.get_batch_with_lang(&user_list, "zh-CN");
//     assert!(res.is_ok());

//     let users = res.unwrap();
//     assert!(users.len() > 0);
// }
