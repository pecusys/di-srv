use serde::{Serialize, Deserialize};
use sqlx::{
    types::{chrono::{Utc, DateTime, NaiveDate, NaiveDateTime}, Json, uuid::Uuid},
    FromRow, Type, postgres::{Postgres, PgRow}, Decode, prelude::*,
};
use crate::{Db,
    models::{Model, User, Status, Visibility, Priority, Item, Group,
        link::{LinkedTo, Link},
    },
};

//TODO add validation so that user/record name combo is unique
#[serde(rename_all="camelCase")]
#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Record {
    #[serde(default="Uuid::new_v4")]
    pub id: Uuid,
    pub uid: Uuid,
    pub name: String,
    #[serde(default="Status::default")]
    pub status: Status,
    #[serde(default="Visibility::default")]
    pub visibility: Visibility,
    #[serde(default="Utc::now")]
    pub created_at: DateTime<Utc>,
}

impl Record {

    pub fn new<U>(uid: Uuid, name: U) -> Self
    where U: Into<String> {
        Self { name: name.into(), uid, ..Self::default() }
    }

    pub fn create<U, V, W>
        (uid: Uuid, name: U, status: V, visibility: W) -> Self
        where  U: Into<String>, V: Into<Status>, W: Into<Visibility> {
        Self {
            name: name.into(), uid,
            status: status.into(),
            visibility: visibility.into(),
            ..Self::default()
        }
    }

    fn build() -> () {}

    pub fn set_name<T>(self, name: T) -> Self where T: Into<String> {
        Self { name: name.into(), ..self }
    }

    pub fn set_status<T>(self, status: T) -> Self where T: Into<Status> {
        Self { status: status.into(), ..self }
    }

    pub fn set_visibility<T>(self, visibility: T) -> Self where T: Into<Visibility> {
        Self { visibility: visibility.into(), ..self }
    }

    pub async fn update_visibility<T>(
        self, db: &Db, visibility: T, id: Uuid,
    ) -> sqlx::Result<Self> where T: Into<Visibility>{
        let vis = visibility.into();
        sqlx::query("UPDATE Records SET visibility=$1 WHERE id=$2")
                .bind(&vis)
                .bind(id)
                .execute(&db.pool).await?;
        Ok ( Self { visibility: vis, ..self } )
    }

    pub async fn update_status<T>(
        self, db: &Db, status: T, id: Uuid,
    ) -> sqlx::Result<Self> where T: Into<Status>{
        let stat = status.into();
        sqlx::query("UPDATE Records SET visibility=$1 WHERE id=$2")
                .bind(&stat)
                .bind(id)
                .execute(&db.pool).await?;
        Ok ( Self { status: stat, ..self } )
    }

    pub async fn get_all(db: &Db) -> sqlx::Result<Vec<Self>> {
        let res: Vec<Record> = sqlx::query_as::<Postgres, Record>(
            "SELECT * FROM Records")
            .fetch_all(&db.pool).await?;
        Ok(res)
    }

    pub async fn get_all_by_user(db: &Db, uid: Uuid) -> sqlx::Result<Vec<Self>> {
        let res: Vec<Record> = sqlx::query_as::<Postgres, Record>(
            "SELECT * FROM Records r WHERE r.uid=$1")
            .bind(uid)
            .fetch_all(&db.pool).await?;
        Ok(res)
    }

    // implemented in model trait -- remove?
    pub async fn get_by_id(db: &Db, id: Uuid) -> sqlx::Result<Option<Self>> {
        let res: Option<Record> = sqlx::query_as::<Postgres, Record>(
            "SELECT * FROM Records WHERE id=$1")
            .bind(id)
            .fetch_optional(&db.pool).await?;
        Ok(res)
    }

    pub async fn get_by_username_and_name(db: &Db, username: String, name: String)
        -> sqlx::Result<Option<Self>>
    {
        let res: Option<Self> = sqlx::query_as::<Postgres, Record>(
            "SELECT r.id, r.name, r.uid, r.status, r.visibility, r.created_at,
             FROM Records r, Users u
             WHERE r.name = $1 AND r.uid = u.id AND u.username = $1")
            .bind(name)
            .bind(username)
            .fetch_optional(&db.pool).await?;
        Ok(res)
    }

    pub async fn insert(self, db: &Db) -> sqlx::Result<Self> {
        let res: Uuid = sqlx::query(
            "INSERT INTO Records (uid, name, status, visibility, created_at)
             VALUES ($1, $2, $3, $4, $5) RETURNING id")
            .bind(&self.uid)
            .bind(&self.name)
            .bind(&self.status)
            .bind(&self.visibility)
            .bind(&self.created_at)
            .fetch_one(&db.pool).await?
            .get("id");
        Ok( Self { id: res, ..self })
    }

    // implemented in linkedto trait -- remove?
    pub async fn get_linked_users(db: &Db, rid: Uuid) -> sqlx::Result<Vec<User>> {
        let res = sqlx::query_as::<Postgres, User>
            ("SELECT u.id, u.username, u.email, u.created_at
              FROM Users u INNER JOIN UserRecordLinks ur ON u.id = ur.uid
                   INNER JOIN Records r on ur.rid = r.id
                   AND r.id = $1")
            .bind(rid)
            .fetch_all(&db.pool)
            .await?;
        Ok(res)
    }

    pub async fn add_new_item<T: Into<String>>
        (db: &Db, rid: Uuid, item_name: T) -> sqlx::Result<Item>
    {
        let item = Item::new(rid, item_name.into()).insert(db).await?;
        let link = Link::new(rid, item.id).insert::<Record, Item>(db).await?;
        Ok(item)
    }

    pub async fn add_existing_item(db: &Db, rid: Uuid, iid: Uuid) -> sqlx::Result<Uuid>
    {
        let link = Link::new(rid, iid).insert::<Record, Item>(db).await?;
        Ok(link)
    }

    pub async fn delete_by_id(db: &Db, id:  Uuid) -> sqlx::Result<Uuid> {
        let res = sqlx::query(
            "DELETE FROM Records WHERE id=$1 RETURNING id")
            .bind(id)
            .fetch_one(&db.pool).await?;
        Ok( res.get("id") )
    }

    pub async fn update_by_id(db: &Db, id: Uuid, record: Record) -> sqlx::Result<Uuid> {
        let res = sqlx::query(
            "DELETE FROM Records WHERE id=$1 RETURNING id")
            .bind(id)
            .fetch_one(&db.pool).await?;
        Ok( res.get("id") )
    }
}

impl Default for Record {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            uid: Uuid::new_v4(),
            name: String::new(),
            status: Status::Active.into(),
            visibility: Visibility::Private.into(),
            created_at: Utc::now(),
        }
    }
}

impl From<Uuid> for Record {
    fn from(uid: Uuid) -> Self {
        Record { uid, ..Record::default() }
    }
}

impl From<User> for Record {
    fn from(user: User) -> Self {
        Record { uid: user.id, ..Record::default() }
    }
}

impl From<&'static PgRow> for Record {
    fn from(row: &'static PgRow) -> Self {
        Record::from_row(row).unwrap()
    }
}

#[async_trait::async_trait]
impl Model for Record {
    fn table() -> String { String::from("Records") }
    fn foreign_id() -> String { String::from("rid") }
    fn id(self) -> Uuid { self.id }
    fn fields() ->  Vec<String> {
        let fields = vec!["id", "uid", "name", "status", "visibility", "created_at"];
        fields.into_iter()
            .map(|field| field.to_string())
            .collect::<Vec<String>>()
    }
}

impl LinkedTo<User> for Record {  }
impl LinkedTo<Item> for Record {  }
impl LinkedTo<Group> for Record {  }
