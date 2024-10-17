// 1. Import the dependencies
// 2. Create the model (a user with Id, name, and email) and add constants
// 3. Main function: database connection and TCP server.
// 4. Create the routes in a function (endpoints) : handle_client
// 5. Utility functions: set_database, get_id, get_user_request_body
// 6. Create the controllers : handle_post_request for the Create endpoint etc for post, get, get all, delete, put


use postgres::{ Client,NoTls };
use postgres::Error as PostgresError;
use std::net::{ TcpListener, TcpStream };
use std::io::{ Read, Write };
use std::env;

#[macro_use]
extern crate serde_derive;

// Client is used to connect to the database.
// NoTls is used to connect to the database without TLS.
// PostgresError is the error type returned by the Postgres driver.
// TcpListener and TcpStream to create a TCP server.
// Read and Write are used to read and write from a TCP stream.
// env is used to read the environment variables.
// the #[macro_use] attribute is used to import the serde_derive macro.
// We will use it to derive our model's Serialize and Deserialize traits.

//Model: User struct with id, name, email - way to represent a user in our application.
#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    // id is an integer and is optional. The reason is that we don't provide the id when we create or update a new user.
    //  The database will generate it for us. But we still want to return the user with an id when we get them.
    name: String,
    email: String,
    // name and email is a string, and it is mandatory. We will use it to store the name of the user.
}

//DATABASE URL
const DB_URL: &str = env!("DATABASE_URL");

//constants
const OK_RESPONSE: &str =
    "HTTP/1.1 200 OK\r\n
    Content-Type: application/json\r\n
    Access-Control-Allow-Origin: *\r\n
    Access-Control-Allow-Methods: GET, POST, PUT, DELETE\r\n
    Access-Control-Allow-Headers: Content-Type\r\n\r\n";

const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

const INTERNAL_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";

// let listener = ... declares a new variable named listener and assigns it the 
// value of the expression on the right-hand side of the =.

// TcpListener::bind(...) is a method call that attempts to bind a TCP listener 
// to a specific address and port.

// format!("0.0.0.0:8080") creates a string that represents the address and port
//  to bind to. In this case, it's binding to all available network interfaces (0.0.0.0) on port 8080.

// .unwrap() is a method call that is used to handle the result of the bind method.
//  If the binding is successful, it returns a TcpListener object. If the binding
//   fails (e.g., because the port is already in use), it returns an error. The
//  unwrap method will panic (abort the program) if the result is an error, and 
//  return the underlying value if the result is Ok.

//main function
fn main() {
    //Set Database - pt 1
    if let Err(_) = set_database() { // if setting up the database gives us an error, then do this
        // (_) is wildcard for all errs
        // let is used as pattern matching here. not  for assignment
        println!("Error setting database");
        return;
    }

    //start server and print port - pt 2
    let listener = TcpListener::bind(format!("0.0.0.0:8080")).unwrap();
    println!("Server listening on port 8080");
    // TcpListener is like a phone that listens for incoming calls from other computers.
    // bind is like setting the phone number to 8080, so others can call us
    // 0.0.0.0 is like saying "anyone can call us, no matter where they are".
    // unwrap is like "make sure everything is cool, or freak out if it's not".

    
    // listener.incoming() is used to get the incoming connections - pt 3
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(eimbba) => { //eimbba is spl name we give to the error arisen
                println!("Unable to connect: {}", eimbba);
            }
        }
    }
}

//handle requests
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        // By using &mut buffer, you're allowing the read method to modify the buffer,
        // which is necessary for it to write data into it. If you just used &buffer,
        // the read method wouldn't be able to modify the buffer, and the data wouldn't be written.
        Ok(size) => {
        // (size) represents the number of bytes that were successfully read from the stream and written into the buffer.
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());
        // &buffer[..size]: Slice of buffer from start to size index.
        // String::from_utf8_lossy(...): Convert byte slice to String, replacing invalid UTF-8 with a replacement character.
        // as_ref(): Return a reference to the resulting String (&str).
        // request.push_str(...): Append the string slice to the end of the request string.

            let (status_line, content) = match &*request { // (*)deref, then (&)ref request
                // PATTERN MATCHING - the pattern is (status_line, content),
                // which is a tuple pattern. This pattern is saying
                // "I expect the value on the right-hand side to be a tuple with two elements, 
                // and I want to bind the first element to status_line and the second element to content".

                // Pattern Matching
                // Tuple Pattern Matching: let (status_line, content) = match &*request { ... }
                // Destructures value into two variables
                // Guarded Pattern Matching: r if r.starts_with("OPTIONS") => ...
                // Matches value r only if condition is true
                // Types of Pattern Matching in Rust : Tuple Pattern Matching, Guarded Pattern Matching, Enum Pattern Matching, Struct Pattern Matching, Literal Pattern Matching

                r if r.starts_with("OPTIONS") => (OK_RESPONSE.to_string(), "".to_string()), // creates a new tuple value separated by "," . "whatever.to_string" converts wahtever to string
                r if r.starts_with("POST /api/rust/users") => handle_post_request(r),
                r if r.starts_with("GET /api/rust/users/") => handle_get_request(r),
                r if r.starts_with("GET /api/rust/users") => handle_get_all_request(r),
                r if r.starts_with("PUT /api/rust/users/") => handle_put_request(r),
                r if r.starts_with("DELETE /api/rust/users/") => handle_delete_request(r),
                // THINGS TO KNOW ABOUT r :
                // Pattern variable in match expression.
                // Bound to &*request.
                // Valid only within match arm.
                // Not declared earlier.
                // Temporary variable.
                _ => (NOT_FOUND.to_string(), "404 not found".to_string()),
            };

            stream.write_all(format!("{}{}", status_line, content).as_bytes()).unwrap();
        }
        Err(e) => eprintln!("Unable to read stream: {}", e),
    }
}

// three following utility functions - 
// 1. set_database connects to the database and creates the users table if it doesn't exist.
// 2. get_id is used to get the id from the request URL.
// 3. get_user_request_body is used to deserialize the user from the request body (without the id)
//    for the Create and Update endpoints.

//db setup - util fn 1
fn set_database() -> Result<(), PostgresError> { //special message at the end, either a happy one (()), or a sad one (PostgresError).
    let mut client = Client::connect(DB_URL, NoTls)?;
    // This line is like trying to open a special door to a secret room! We're using a 
    // special key called Client to connect to a place called DB_URL. 
    // The NoTls is like a special permission to enter the room. 
    // The ? at the end is like a special helper that makes sure we can enter the room. If we can't, it will tell us why!
    
    client.batch_execute( // CLIENT IS THE SPECIAL KEY AND BATCH_EXECUTE IS THE BUTTON TO DO SOME WORK IN THE SECRET ROOM
        "
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL
        )
    "
    )?; // ? TELLS US IF FAILED THEN WHY
    Ok(()) // RETURN/SAY "YEAH YOU DID A GOOD JOB"
}

//Get id from request URL - util fn 2
fn get_id(request: &str) -> &str {
    request.split("/").nth(4).unwrap_or_default().split_whitespace().next().unwrap_or_default() // return this - the rust way

    // nth(4): This means we want to find the 5th piece of the puzzle (because we start counting from 0).

    // unwrap_or_default(): This is like a safety net. 
    // If there's no 5th piece (maybe the sentence is too short), it will give us a default value instead of getting upset.

    // split_whitespace(): This splits the piece we found ("my") into even smaller pieces, but only if there are spaces.
    //  If there are no spaces, it will just give us the original piece.

    // next(): This gives us the first piece of the smaller pieces we just made. If there are no pieces,
    //  it will give us a default value again.

    // unwrap_or_default(): Another safety net, just in case!
}

//deserialize user from request body without id - util fn 3
fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> { // Result(...) is type of serde_json
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default()) // return this - the rust way
    // last().unwrap_or_default(): This takes the message we found and makes sure it's the 
    // right one. If there's no message, it will give us a default one instead of getting upset.

    // serde_json::from_str(...): This is like a special decoder ring that helps us 
    // understand the secret code. It takes the message and turns it into something we 
    // can read and understand.

    //  But, there's a catch! If the decoder ring can't understand the message,
    //  it will send us an error message instead. That's why we have 
    //  Result<User, serde_json::Error>.
    //  you won't get two messages at the same time, but you might get one of 
    //  these two messages depending on how well the function can read the secret code.
}

//handle post request
fn handle_post_request(request: &str) -> (String, String) {
    match (get_user_request_body(request), Client::connect(DB_URL, NoTls)) {
        (Ok(user), Ok(mut client)) => {
                            // Insert the user and retrieve the ID
                            let row: postgres::Row = client.query_one("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",&[&user.name, &user.email]).unwrap(); // inner (&) creates refs, outer (&) creates ref of (arr of refs)
                            // client.query_one: Expects exactly one row from the query; errors if there's none or more than one.
                            // client.query: Handles multiple rows, returning a list (even if empty) without errors.
                            let user_id: i32 = row.get(0);
                            // Fetch the created user data
                            match client.query_one("SELECT id, name, email FROM users WHERE id = $1", &[&user_id]) {
                                Ok(row) => {
                                                let user = User {
                                                    id: Some(row.get(0)),
                                                    name: row.get(1),
                                                    email: row.get(2),
                                                };
                                                (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap()) // return this - the rust way
                                            }
                                Err(_) => (INTERNAL_ERROR.to_string(), "Failed to retrieve created user".to_string()), // In Rust, a trailing comma is allowed in a tuple with at least two elements, like this: let my_tuple = ("hello", 42,);.
                            }
                        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

//handle get request
fn handle_get_request(request: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(DB_URL, NoTls)) {
        (Ok(id), Ok(mut client)) =>
            match client.query_one("SELECT * FROM users WHERE id = $1", &[&id]) {
                Ok(row) => {
                    let user = User {
                        id: row.get(0),
                        name: row.get(1),
                        email: row.get(2),
                    };

                    (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap())
                }
                _ => (NOT_FOUND.to_string(), "User not found".to_string()),
            }

        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

//handle get all request
fn handle_get_all_request(_request: &str) -> (String, String) {
    match Client::connect(DB_URL, NoTls) {
        Ok(mut client) => {
            let mut users = Vec::new();

            for row in client.query("SELECT id, name, email FROM users", &[]).unwrap() {
                users.push(User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                });
            }

            (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap())
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

//handle put request
fn handle_put_request(request: &str) -> (String, String) {
    match
        (
            get_id(&request).parse::<i32>(),
            get_user_request_body(&request),
            Client::connect(DB_URL, NoTls),
        )
    {
        (Ok(id), Ok(user), Ok(mut client)) => {
            client
                .execute(
                    "UPDATE users SET name = $1, email = $2 WHERE id = $3",
                    &[&user.name, &user.email, &id]
                )
                .unwrap();

            (OK_RESPONSE.to_string(), "User updated".to_string())
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

//handle delete request
fn handle_delete_request(request: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(DB_URL, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            let rows_affected = client.execute("DELETE FROM users WHERE id = $1", &[&id]).unwrap();

            //if rows affected is 0, user not found
            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "User not found".to_string());
            }

            (OK_RESPONSE.to_string(), "User deleted".to_string())
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}