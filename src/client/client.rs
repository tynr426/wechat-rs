use std::io::Read;
use std::collections::HashMap;

use url::Url;
// use hyper::{self, Client,Body, Request};

use rustc_serialize::json::{self, Json, Object};
use rustc_serialize::Encodable;

use errors::WeChatError;
use types::{WeChatResult};
use session::SessionStore;
use utils::current_timestamp;
use reqwest::header;

const REFETCH_ACCESS_TOKEN_ERRCODES: [i32; 3] = [40001, 40014, 42001];


#[derive(Debug, Clone)]
pub struct APIClient<T: SessionStore> {
    pub appid: String,
    pub secret: String,
    pub session: T,
}

impl<T: SessionStore> APIClient<T> {

    #[inline]
    pub fn new<S: Into<String>>(appid: S, secret: S, session: T) -> APIClient<T> {
        APIClient {
            appid: appid.into(),
            secret: secret.into(),
            session: session,
        }
    }

    #[inline]
    pub fn with_access_token<S: Into<String>>(appid: S, secret: S, access_token: S, session: T) -> APIClient<T> {
        let appid = appid.into();
        let secret = secret.into();
        let client = Self::new(appid.clone(), secret, session);
        let token_key = format!("{}_access_token", appid);
        client.session.set(&token_key, access_token.into(), None);
        client
    }

    #[inline]
    pub fn access_token(&self) -> String {
        let token_key = format!("{}_access_token", self.appid);
        let expires_key = format!("{}_expires_at", self.appid);
        let token: String = self.session.get(&token_key, Some("".to_owned())).unwrap();
        let timestamp = current_timestamp();
        let expires_at: i64 = self.session.get(&expires_key, Some(timestamp)).unwrap();
        if expires_at <= timestamp {
            "".to_owned()
        } else {
            token
        }
    }

    pub fn request<D: Encodable>(&self, method: &str, url: &str, params: Vec<(&str, &str)>, data: &D) -> WeChatResult<String> {
        let mut http_url = if !(url.starts_with("http://") || url.starts_with("https://")) {
            let url_string=format!("https://api.weixin.qq.com/cgi-bin/{}",url);
            Url::parse(&url_string).unwrap()
        } else {
            Url::parse(url).unwrap()
        };
        let access_token = self.access_token();
        let mut querys = params.clone();
        if !access_token.is_empty() {
            debug!("Using access_token: {}", access_token);
            querys.push(("access_token", &access_token));
        }
        &http_url.query_pairs_mut().extend_pairs(querys.into_iter());
        println!("url:{:?}",http_url.as_str());
        if method.to_lowercase()=="post"{
            let body = match json::encode(data) {
                Ok(text) => text,
                Err(_) => "".to_owned(),
            };
            post(http_url.as_str(),&body)
        }
        else{
            get(http_url.as_str())
        }

        // let client = Client::new();
        // let req = Request::builder()
        // .method(method)
        // .uri(http_url.as_str())
        // .body(Body::from(body))
        // .expect("request builder");
       // Ok(client.request(req))

    //    let futures_response=client.request(req);
    //    //hyper::body::to_bytes(futures_response)

    //     let fut = futures_response.then(|_res|{
    //         _res.into_body().concat2()
    //     }).and_then(|body|{
    //         let s = ::std::str::from_utf8(&body.as_bytes().to_vec()).expect("httpbin sends utf-8 JSON");
    //         Ok(())

    //     }) .map_err(|err| {
    //         println!("error: {}", err);
    //     });
    //     //.map_err(|e| Err(WeChatError::ClientError { errcode: -1, errmsg: format!("Send request error: {}", e) }))
    //     //.map(Ok)
        
        
    //     fut

    }

    fn _upload_file<R: Read>(&self, url: &str, params: Vec<(&str, &str)>, files: &mut HashMap<String, &mut R>) -> WeChatResult<String> {
        let mut http_url = if !(url.starts_with("http://") || url.starts_with("https://")) {
            let mut url_string = "https://api.weixin.qq.com/cgi-bin/".to_owned();
            url_string = url_string + url;
            Url::parse(&url_string).unwrap()
        } else {
            Url::parse(url).unwrap()
        };
        let access_token = self.access_token();
        let mut querys = params.clone();
        if !access_token.is_empty() {
            debug!("Using access_token: {}", access_token);
            querys.push(("access_token", &access_token));
        }
        http_url.query_pairs_mut().extend_pairs(querys.into_iter());

        // let request = Request::new(Method::Post, http_url).unwrap();
        // let mut req = Multipart::from_request(request).unwrap();
        // for (name, stream) in files.iter_mut() {
        //     // TODO: error handing
        //     req.write_stream(name, stream, None, None).unwrap();
        // }
        // let res = match req.send() {
        //     Ok(_res) => _res,
        //     Err(ref e) => {
        //         error!("Send request error");
        //         return Err(WeChatError::ClientError { errcode: -1, errmsg: format!("Send request error: {}", e) });
        //     }
        // };
  
        return Err(WeChatError::ClientError { errcode: -1, errmsg: format!("Send request error") });
    }

    pub fn upload_file<R: Read>(&self, url: &str, params: Vec<(&str, &str)>, files: &mut HashMap<String, &mut R>) -> WeChatResult<Json> {
        if self.access_token().is_empty() {
            self.fetch_access_token();
        }
        let res = self._upload_file(url, params.clone(), files)?;
        let data = match self.json_decode(&res) {
            Ok(_data) => _data,
            Err(err) => {
                if let WeChatError::ClientError { errcode, .. } = err {
                    if REFETCH_ACCESS_TOKEN_ERRCODES.contains(&errcode) {
                        // access_token expired, fetch a new one and retry request
                        self.fetch_access_token();
                        let res1 = self._upload_file(url, params, files)?;
                        self.json_decode(&res1)?
                    } else {
                        return Err(err);
                    }
                } else {
                    return Err(err);
                }
            },
        };
        Ok(data)
    }

    #[inline]
    fn json_decode(&self, data:&str) -> WeChatResult<Json> {
        let obj = match Json::from_str(data) {
            Ok(decoded) => { decoded },
            Err(ref e) => {
                error!("Json decode error");
                return Err(WeChatError::ClientError { errcode: -3, errmsg: format!("Json decode error: {}", e) });
            }
        };
        match obj.find("errcode") {
            Some(code) => {
                let errcode = code.as_i64().unwrap();
                if errcode != 0 {
                    let errmsg = match obj.find("errmsg") {
                        Some(msg) => {
                            msg.as_string().unwrap()
                        },
                        None => { "" }
                    };
                    return Err(WeChatError::ClientError { errcode: errcode as i32, errmsg: errmsg.to_owned() });
                }
            },
            None => {},
        };
        Ok(obj)
   
    }

    #[inline]
    pub fn post<D: Encodable>(&self, url: &str, params: Vec<(&str, &str)>, data: &D) -> WeChatResult<Json> {
        if self.access_token().is_empty() {
            self.fetch_access_token();
        }
        let res = self.request("POST", url, params.clone(), data)?;
        let data = match self.json_decode(&res) {
            Ok(_data) => _data,
            Err(err) => {
                if let WeChatError::ClientError { errcode, .. } = err {
                    if REFETCH_ACCESS_TOKEN_ERRCODES.contains(&errcode) {
                        // access_token expired, fetch a new one and retry request
                        self.fetch_access_token();
                        let res1 = self.request("POST", url, params, data)?;
                        self.json_decode(&res1)?
                    } else {
                        return Err(err);
                    }
                } else {
                    return Err(err);
                }
            },
        };
        Ok(data)
    }

    #[inline]
    pub fn get(&self, url: &str, params: Vec<(&str, &str)>) -> WeChatResult<Json> {
        if self.access_token().is_empty() {
            self.fetch_access_token();
        }
        let res = self.request("GET", url, params.clone(), &Object::new())?;
        let data = match self.json_decode(&res) {
            Ok(_data) => _data,
            Err(err) => {
                if let WeChatError::ClientError { errcode, .. } = err {
                    if REFETCH_ACCESS_TOKEN_ERRCODES.contains(&errcode) {
                        // access_token expired, fetch a new one and retry request
                        self.fetch_access_token();
                        let res1 = self.request("GET", url, params, &Object::new())?;
                        self.json_decode(&res1)?
                    } else {
                        return Err(err);
                    }
                } else {
                    return Err(err);
                }
            },
        };
        Ok(data)
    }

    pub fn fetch_access_token(&self) -> Option<String> {
        let res = self.request(
            "GET",
            "token",
            vec![
                ("grant_type", "client_credential"),
                ("appid", &self.appid),
                ("secret", &self.secret),
            ],
            &Object::new()
        );
        
        let raw_data = match res {
            Ok(raw) => raw,
            Err(_) => { return None; },
        };
        let data = match self.json_decode(&raw_data) {
            Ok(data) => data,
            Err(_) => { return None; },
        };
        let token = match data.find("access_token") {
            Some(token) => token,
            None => { return None; }
        };
        let expires_in = match data.find("expires_in") {
            Some(expires) => expires.as_u64().unwrap() as usize,
            None => 7200usize,
        };
        let expires_at = current_timestamp() + expires_in as i64;
        let token_str = match *token {
            Json::String(ref v) => {
                let token_key = format!("{}_access_token", self.appid);
                let expires_key = format!("{}_expires_at", self.appid);
                self.session.set(&token_key, v.to_owned(), Some(expires_in));
                self.session.set(&expires_key, expires_at, Some(expires_in));
                Some(format!("{}", v))
            },
            _ => None,
        };
        println!("token={:?}",token_str);
        token_str
    }
}

// 默认user_agent
const DEFAULT_USER_AGENT: &'static str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3534.4 Safari/537.36";
// Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.5 Safari/605.1.15


//采用post方式请求数据
pub(crate) fn post(url: &str, params: &str) -> Result<String, WeChatError> {
    match reqwest::blocking::Client::new()
            .post(url)
            .header(header::USER_AGENT, DEFAULT_USER_AGENT)
            .form(params)
            .send()
        {
            Ok(res) => {
                if res.status() == 200 {
                    match res.text() {
                        Ok(txt) => Ok(txt),
                        Err(e) =>Err(WeChatError::ClientError { errcode: -1, errmsg: format!("Send request error: {}", e) })
                    }
                } else {
                    Err(WeChatError::ClientError { errcode: 500, errmsg: format!("status={}",res.status()) })
                }
            },
            Err(e)=>Err(WeChatError::ClientError { errcode: 500, errmsg: format!("Send request error: {}", e) })
        }
}

pub(crate)  fn get(url: &str) -> Result<String, WeChatError> {
    // let mut headers = header::HeaderMap::new();
    // headers.insert(header::HeaderName::from_static("UserAgent"), header::HeaderValue::from_static(DEFAULT_USER_AGENT));
    

    match reqwest::blocking::Client::new()
        .get(url)
        .header(header::USER_AGENT, DEFAULT_USER_AGENT)
        .send()
        {
            Ok(res) => {
                if res.status() == 200 {
                    match res.text() {
                        Ok(txt) => Ok(txt),
                        Err(e) =>Err(WeChatError::ClientError { errcode: -1, errmsg: format!("Send request error: {}", e) })
                    }
                } else {
                    Err(WeChatError::ClientError { errcode: 500, errmsg: format!("status={}",res.status()) })
                }
            },
            Err(e)=>Err(WeChatError::ClientError { errcode: 500, errmsg: format!("Send request error: {}", e) })
        }
}