#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate diesel;
#[macro_use] extern crate serde_derive;
extern crate rocket;
extern crate rocket_contrib;
extern crate exo;

use exo::models::*;
use exo::db_connection::DbConn;
use exo::db_connection::init_db_contection_pool;
use diesel::prelude::*;
use rocket_contrib::Json;
use rocket::request::Form;

#[cfg(test)] mod tests;

//#[get("/tasks")]
//fn get_tasks(conn: DbConn) -> QueryResult<Json<Vec<Task>>> {
//    all_tasks.order(tasks::id.desc())
//        .load::<Task>(&*conn)
//        .map(|tasks| Json(tasks))
//}

mod api {
    #[derive(Serialize)]
    pub struct TicketHead {
        pub project: String,
        pub number: i32,
        pub title: String,
    }

    #[derive(FromForm)]
    pub struct CreateTicket {
        pub title: String,
        pub body: String
    }

    #[derive(Serialize)]
    pub struct CreatedTicket {
        pub project: String,
        pub number: i32,
        pub state: i32
    }
}





#[get("/")]
fn index(conn: DbConn) -> Json<Vec<api::TicketHead>> {
    use exo::schema::tickets::dsl::*;

    let mut results = tickets.load::<Ticket>(&conn.0).expect("Error loading posts");
    let heads = results.drain(..).map(|ticket| api::TicketHead {
        project: ticket.project.to_string(),
        number: ticket.number,
        title: ticket.title
    }).collect();

    Json(heads)
}

#[post("/", data = "<ticket>")]
fn create_ticket(conn: DbConn, ticket: Form<api::CreateTicket>) -> Json<api::CreatedTicket> {
    use exo::schema::tickets::dsl::*;
    use diesel::dsl::*;

    let max_number: Option<i32> = tickets
        .filter(project.eq(0))
        .select(max(number))
        .first::<Option<i32>>(&conn.0)
        .expect("Error loading max ticket_number");

    let new_number = max_number.map(|no| no + 1).unwrap_or(0);
    let api::CreateTicket { title: xtitle, body: xbody } = ticket.get();
    let new_ticket = NewTicket {
        project: 0,
        number: new_number,
        state: 0,
        title: xtitle.to_string(),
        body: xbody.to_string(),
        ctime: 0
    };
    // TODO: handle same ticket number
    diesel::insert_into(tickets).values(&new_ticket).execute(&conn.0).expect("Inserting failed");

    Json(api::CreatedTicket {
        project: 0.to_string(),
        number: new_number,
        state: 0
    })
}

#[get("/")]
fn ticket(_conn: DbConn) -> String {
    /*use exo::schema::ticket_attributes::dsl::*;

    let results = ticket_attributes.load::<Ticket>(&conn.0).expect("Error loading posts");
    let mut res: String = format!("Displaying {} tickets:", results.len());
    for ticket in results {
        res += &format!("PROJECT{}-{}: {}", ticket.project, ticket.number, ticket.title);
    }
    res*/
    String::new()
}

fn main() {
    rocket::ignite()
        .manage(init_db_contection_pool())
        .mount("/", routes![index])
        .mount("/ticket", routes![create_ticket])
        .launch();
}