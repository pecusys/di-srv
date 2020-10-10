use actix_web::client::Client;
use actix_redis::RedisSession;
use actix_session::Session;
use serde::{Serialize, Deserialize};
use crate::{state::State, models::UserIn};
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{ Error, 
    http::{Cookie, HeaderName, HeaderValue},
    web::{self, delete, get, post, put, resource, scope, ServiceConfig},
    HttpRequest, HttpResponse, Scope,
};
use com::auth::Auth;
use divdb::models::user::*;

#[derive(Serialize, Deserialize)]
pub struct CognitoIn {}

pub fn routes() -> Scope {
    scope("/auth")
        .service(cognito_routes())
        .service(resource("")
            .route(get().to(|| HttpResponse::Ok().body("GET /auth")))
            .route(get().to(|| HttpResponse::Ok().body("POST /auth")))
            .route(get().to(|| HttpResponse::Ok().body("DELETE /auth"))),
        )
        .service(resource("/login")
            .route(get().to(|| HttpResponse::Ok().body("GET /auth/login")))
            .route(post().to(login)),
        )
        .service(resource("/signup")
            .route(get().to(|| HttpResponse::Ok().body("GET /auth/signup")))
            .route(post().to(signup)),
        )
        .service(resource("/logout")
            .route(get().to(logout_session))
            .route(post().to(logout)),
        )
        .service(resource("/refresh")
            .route(get().to(check_id))
            .route(post().to(refresh_login)),
        )
        .service(resource("/check")
            .route(get().to(check_session))
        )
}

pub fn cognito_routes() -> Scope {
        scope("/cognito")
            .service(resource("/login").route(get().to(cognito_login)))
            .service(resource("/logout").route(get().to(cognito_logout)))
            .service(resource("/signup").route(get().to(cognito_signup)))
            .service(resource("/authorize").route(get().to(cognito_authorize)))
            .service(resource("/token").route(get().to(cognito_token)))
}

pub fn validate(session: &Session) -> Result<i32, actix_web::HttpResponse> {
    let uid: Option<i32> = session.get("id").unwrap_or(None);
    match uid {
        Some(uid) => { session.renew(); Ok(uid) },
        None => Err(HttpResponse::Unauthorized().json("Unauthorized"))
    }
}

pub async fn signup(
    (req, user, data): (HttpRequest, web::Json<User>, web::Data<State>),
) -> HttpResponse {
    //let user = user.clone();
    let hashed_user = User {
        password: Auth::new().hash(&user.password).unwrap(), ..user.clone()
    };
    println!("SIGNUP: {}", serde_json::to_string(&hashed_user).unwrap());
    match hashed_user.insert(&data.db.lock().unwrap()).await {
        Ok(_uid) => {
            HttpResponse::Ok()
                .body("User signed up")
        },
        Err(_) => HttpResponse::NotAcceptable().finish()
    }
}

pub async fn login(
    (id, session, req, user, data): 
    (Identity, Session, HttpRequest, web::Json<UserLogin>, web::Data<State>),
) -> Result<HttpResponse, HttpResponse> {
    let user = user.into_inner().clone();
    match User::get_by_username(&data.db.lock().unwrap(), user.username).await {
        Ok(Some(db_user)) => {
            if Auth::new().verify(user.password, &db_user.password).unwrap() {
                let user_in = UserIn::from(db_user.clone());
                let login_str = serde_json::to_string(&user_in).unwrap();
                id.remember(login_str.clone());
                session.set("uid", &db_user.id.unwrap()).unwrap();
                Ok(HttpResponse::Ok()
                    .set_header("authorization", "true")
                    .content_type("application/json")
                    .json(&user_in))
            } else { Err(HttpResponse::NotFound().body("Couldn't login")) }
        }
        _ => Err(HttpResponse::NotFound().body("COuldn't login"))
    }
}

pub async fn logout(id: Identity, session: Session) -> HttpResponse {
    match id.identity() {
        Some(_ident) => {
            id.forget();
            session.purge();
            HttpResponse::Ok()
                .set_header("authorization", "false")
                .body("User logged out")
        }
        None => HttpResponse::NotFound().body("No user to log out"),
    }
}

pub async fn logout_session(session: Session) -> Result<HttpResponse, HttpResponse> {
    let sess: Result<Option<i32>, Error> = session.get("uid");
    match sess {
        Ok(Some(uid)) => { 
            session.remove("uid");
            Ok(HttpResponse::Ok()
                .set_header("authorization", "false")
                .body("User logged out"))
        }
        _ => Err(HttpResponse::NotFound().body("No user to log out")),
    }
}

pub async fn refresh_login(
    (id,  data): (Identity, web::Data<State>)) -> HttpResponse 
{
    match id.identity() {
        Some(id) => {
            println!("REFRESH: {}", id);
            let user: UserIn = serde_json::from_str(&id).unwrap();
            HttpResponse::Ok()
                .set_header("authorization", "true")
                .json(&user)
        },
        None => HttpResponse::Gone()
            .set_header("authorization", "false")
            .json(false)
        
    }
}

pub async fn check_id(
    (id, req, user, data): (
        Identity,
        HttpRequest,
        web::Json<UserLogin>,
        web::Data<State>,
    ),
) -> HttpResponse {
    match id.identity() {
        Some(id) => {
            let user: UserIn = serde_json::from_str(&id).unwrap();
            HttpResponse::Ok()
                .set_header("authorization", "true")
                .json(&user)
        }
        None => HttpResponse::NotFound()
            .set_header("authorization", "false")
            .json(false)
    }
}

pub async fn check_session(
    (session, req, user, data): (
        Session,
        HttpRequest,
        web::Json<UserLogin>,
        web::Data<State>,
    ),
) -> Result<HttpResponse, HttpResponse> {
    let sess: Result<Option<i32>, Error> = session.get("uid");
    match sess {
        Ok(Some(uid)) => {
            Ok(HttpResponse::Ok()
                .json(uid))
        }
        _ => Err(HttpResponse::NotFound()
                .json(false))
    }
}

pub async fn cognito_login(
    (req,  data, body): (HttpRequest,web::Data<State>, web::Json<CognitoIn>) ) -> HttpResponse 
{
    let payload = body.into_inner();
    match Client::new()
        .get("https://in.div.is/oauth2/login")
        .send_json(&payload).await {
        Ok(res) => HttpResponse::Ok()
            .content_type("application/json")
            .json(true),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

pub async fn cognito_authorize(
    (req,  data, body): (HttpRequest,web::Data<State>, web::Json<CognitoIn>) ) -> HttpResponse 
{
    let payload = body.into_inner();
    match Client::new()
        .get("https://in.div.is/oauth2/authorize")
        .send_json(&payload).await {
        Ok(res) => HttpResponse::Ok()
            .content_type("application/json")
            .json(true),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

pub async fn cognito_logout(
    (req,  data, body): (HttpRequest, web::Data<State>, web::Json<CognitoIn>) ) -> HttpResponse 
{
    let payload = body.into_inner();
    match Client::new()
        .get("https://in.div.is/oauth2/logout")
        .send_json(&payload).await {
        Ok(res) => HttpResponse::Ok()
            .content_type("application/json")
            .json(true),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

pub async fn cognito_signup(
    (req,  data, body): (HttpRequest,web::Data<State>, web::Json<CognitoIn>) ) -> HttpResponse 
{
        HttpResponse::NotFound()
            .set_header("authorization", "false")
            .json(false)
}

pub async fn cognito_userinfo(
    (req,  data, body): (HttpRequest,web::Data<State>, web::Json<CognitoIn>) ) -> HttpResponse 
{
    let payload = body.into_inner();
    match Client::new()
        .get("https://in.div.is/oauth2/userinfo")
        .send_json(&payload).await {
        Ok(res) => HttpResponse::Ok()
            .content_type("application/json")
            .json(true),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

pub async fn cognito_token(
    (req,  data, body): (HttpRequest,web::Data<State>, web::Json<CognitoIn>), ) -> HttpResponse 
{
    let payload = body.into_inner();
    let res = Client::new()
        .post("https://in.div.is/oauth2/token")
        .send_json(&payload)
        .await;
    match res {
        Ok(res) => HttpResponse::Ok()
            .content_type("application/json")
            .json(true),
        Err(_) => HttpResponse::NotFound().finish()
    }
}
