use schema::tickets;

#[derive(Queryable)]
pub struct Ticket {
    pub id: Option<i32>,
    pub project: i32,
    pub number: i32,
    pub state: i32,
    pub title: String,
    pub body: String,
    pub ctime: i32,
}

#[derive(Queryable)]
pub struct Projects {
    pub id: i32,
    pub project: i32,
    pub name: String,
    pub ctime: i32
}

#[derive(Queryable)]
pub struct Attribute {
    pub name: String,
    pub value: String
}

#[derive(Insertable)]
#[table_name = "tickets"]
pub struct NewTicket {
    pub project: i32,
    pub number: i32,
    pub state: i32,
    pub title: String,
    pub body: String,
    pub ctime: i32,
}
