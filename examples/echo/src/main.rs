// Copyright
//! Licensed under the
use actix_web::{web, App, HttpServer,middleware,HttpRequest, HttpResponse,Error};
use futures::{Future, Stream};

use actix_files;
use actix_cors::Cors;
use std::collections::{HashMap, BTreeMap};
// use actix_web::{HttpRequest};
use json::{JsonValue};



fn main() -> std::io::Result<()> {
    // 获取输入的参,参数为待处理的路径
    let args: Vec<String>= std::env::args().collect();

    // 获取数据来源参数
    let mode = if args.len() > 1 {
        &args[1]
    }
    else{
        ""
    };

    // println!("{}", "web_conf");

    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=trace");
    env_logger::init();
    // let log_conf_path = "config/log.yaml";
    // Logs::new().verify_log_config(log_conf_path);
    // // 日志系统初始化
    // log4rs::init_file(log_conf_path, Default::default()).unwrap();

    // 新建一个服务处理
    let sys = actix_rt::System::new("xfs-server");

    let addr = format!("{}:{}", "127.0.0.1", "3003");
    println!("site {} started", addr);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new()
                .header("X-Version", "0.1")
                .header("sever", "das-server")
            )
            .wrap( 
                Cors::default() // 设置允许跨域请求
            )
            .service(
                web::scope("/callback")
                // .wrap(
                //     middleware::DefaultHeaders::new()
                //         .header("X-Version", "0.1")
                //         .header("access-control-allow-headers", "*")
                //         .header("access-control-allow-methods", "POST")
                //         .header("access-control-allow-origin", "*"),
                // )
                .route("", web::get().to_async(api_get))
                .route("", web::post().to_async(api_post)),
                    
            )
            .service(
                // static files
                actix_files::Files::new("/", "/wwww").index_file("index.html")
            )
            
    })
    .bind(addr)?
    // .workers(1) // 指定启用的工作线程数
    .start();
    
    sys.run()
}
// 处理get方式的请求
pub fn api_get(_req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    // println!("{:?}", req);
    // let api_res = distribute("ss", &client);

    Box::new(Ok::<_, Error>(
        HttpResponse::Ok()
            .content_type("application/json")
            .body(r#"{"error": "not support the GET method"}"#),
    ))
}

// 处理post方式请求
pub fn api_post(req: HttpRequest,payload: web::Payload) -> impl Future<Item = HttpResponse, Error = Error> {
    // println!("{:?}", req);
    
    payload
        .map_err(Error::from)
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok::<_, Error>(body)
        })
        .and_then(move |body| {
            let some_val = req.head().headers.get("content-type");
            let content_type = match some_val {
                Some(t) => {
                    t.to_str().unwrap().to_string()
                },
                None => "".to_owned()
            };
            let x_str = std::str::from_utf8(&body);
            // println!("{:?}, {:?}", content_type, x_str);
            let mut p = get_parameters(&content_type, x_str.unwrap());
            // p.add_property("ip", remote_ip.clone());
            // let ip = remote_ip.clone();
            // println!("{:?}",ip);
            let res= Ok(200);
            // let res = if p
            //     .set_secret("react_secret")
            //     // .set_addr(remote_ip.clone())
            //     .set_debug(true)
            //     .validate()
            // {
            //     distribute::distribute(p, req)
            // } else {
            //     get_error_string("签名错误", 40002)
            // };

            // // 对josn格式数据进行校验
            // let result = json::parse(&res);
            // let json_res: JsonValue = match result {
            //     Ok(v) => v,
            //     Err(e) => json::object! {"err" => e.to_string() },
            // };

            // println!("{:?}", res);

            Ok::<_, Error>(
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(res),
            )
        })
}
 pub fn get_parameters(content_type: &str, form_str: &str) -> HashMap {
        let mut value_maps = HashMap::new();
        // 对传入的json格式数据进行处理
        if content_type == "application/json" || content_type.contains("text/plain") {
            let result = json::parse(form_str); // return Result
            let in_json: JsonValue = match result {
                Ok(v) => v,
                Err(e) => json::object! {"err" => e.to_string() },
            };
            for (key, value) in in_json.entries() {
                let val = if value.is_string() {
                    match value{
                        Some(v) => v,
                        None => "".to_owned()
                    }
                }
                else{
                    value.dump()
                };
                value_maps.insert(key.to_owned(), val);
            }
        }
        else{ // 对表单格式数据进行处理
            for xs in String::from(form_str).split("&") {
                let xxs: Vec<&str> = xs.split("=").collect();
                value_maps.insert(xxs[0].to_owned(), xxs[1].to_owned());
            }
        }
        value_maps
    }


