use actix_web::{web, App, HttpServer,HttpRequest,Responder,HttpResponse};
use actix_cors::Cors;
use mongodb::{options::ClientOptions, Client, bson::doc, options::FindOptions};
extern crate dotenv;
use futures::stream::TryStreamExt;
use dotenv::dotenv;
use std::env;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use bson::{ oid::ObjectId, Bson};



#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  dotenv().ok();
  let db_url = env::var("DATABASE_URI").expect("Mongo uri must be provided.");
  let port = match env::var("PORT"){
    Ok(port)=>port,
    Err(_)=>"3000".to_string()
  };
  println!("Server Running at {} ....", port);

  let client_options = ClientOptions::parse( &db_url).await.expect("error");
  let client = web::Data::new(Mutex::new(Client::with_options(client_options).unwrap()));

  HttpServer::new(move || {
    let cors = Cors::default()
    .allow_any_header()
    .allow_any_origin()
    .allow_any_method();

    App::new()
      .wrap(cors)
      .app_data(client.clone())
      .route("/hello/{name}", web::get().to(greet))
      .route("/hello", web::get().to(greet))
      .route("/articles", web::get().to(get_articles))
      .route("/articles/{id}/comments", web::get().to(get_comments))
      .route("/articles/{id}/comments", web::post().to(new_comment))
  })
  .bind("127.0.0.1:".to_owned() + &port)?
  .run()
  .await
}


async fn greet(req: HttpRequest) -> impl Responder {
  println!("{:?}", &req);
  let name = req.match_info().get("name").unwrap_or("World");
  format!("Hello {}!", &name)
}

/* async fn add_article( data:web::Data<Mutex<Client>> ) -> impl Responder{
  let db = data.lock().unwrap().database("main");
  let articles = db.collection("articles"); 
  articles.insert_one(doc!{
    "id": 6,
    "created_at": "2021-10-16T09:00:26.000000Z",
    "updated_at": "2021-10-23T08:21:19.000000Z",
    "name": "Curabitur nec suscipit lorem",
    "shortDesc": "Duis sit amet ligula hendrerit, mattis libero vel, facilisis nisi",
    "date": "06.09.2021",
    "desc": "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.",
    "preview_image": "preview.jpg",
    "full_image": "full.jpeg",
    "category": 2,
    "slider": true
    }, None).await.expect("error");
  format!("success") 
} */
#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
  pub id:ObjectId,
  pub created_at:String,
  pub updated_at:String,
  pub name:String,
  pub shortDesc:String,
  pub date:String,
  pub preview_image:String,
  pub full_image:String,
  pub category:i32,
  pub slider:bool,
}


async fn get_articles(data: web::Data<Mutex<Client>>) -> impl Responder{
  let articles = data
  .lock()
  .unwrap()
  .database("main")
  .collection::<Bson>("articles");
  
  let filter = doc!{};
  let find_options = FindOptions::builder().sort(doc! { "_id": -1}).build(); 
  let mut cursor = articles.find(filter, find_options).await.unwrap();
  
  let mut results = Vec::new();
  while let Some(result) = cursor.try_next().await.unwrap() {
    results.push(result)
  }
  HttpResponse::Ok().json(results) 
}

async fn get_comments(req: HttpRequest,data: web::Data<Mutex<Client>>) -> HttpResponse {
  let article_id = req.match_info().get("id").unwrap().parse::<i32>().unwrap();

  let comments = data
  .lock()
  .unwrap()
  .database("main")
  .collection::<Bson>("сomments");
  
  let mut cursor = comments.find( doc!{"article_id": article_id}, None).await.unwrap();
  
  let mut results = Vec::new();
  while let Some(result) = cursor.try_next().await.unwrap() {
    results.push(result)
  }

  HttpResponse::Ok().json(results)
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
  pub user_name:String,
  pub comment:String,
  pub article_id:i32,
}

#[derive(Serialize, Deserialize)]
pub struct CommentIncome {
  pub user_name:String,
  pub comment:String,
}
async fn new_comment(comment_income: web::Json<CommentIncome>, req: HttpRequest, data: web::Data<Mutex<Client>>) -> HttpResponse {
  let db = data.lock().unwrap().database("main");
  let comments = db.collection("сomments"); 
  let article_id = req.match_info().get("id").unwrap().parse::<i32>().unwrap();

  comments.insert_one(doc!{"user_name":format!("{}", comment_income.user_name), "comment":format!("{}", comment_income.comment), "article_id":article_id}, None).await.expect("error");
  HttpResponse::Ok().json( doc!{"user_name":format!("{}", comment_income.user_name), "comment":format!("{}", comment_income.comment), "article_id":article_id})
}