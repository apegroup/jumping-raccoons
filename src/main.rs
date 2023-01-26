#[macro_use]
extern crate rouille;

// use std::io::Read;
use std::thread;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use tokio::time::Instant;

use lazy_static::lazy_static;
use rouille::{Response, websocket};

const ZAPIER_URL: &'static str = "https://hooks.zapier.com/hooks/catch/128523/bvffe6n/";
const CALLBACK_URL: &'static str = "https://4fcf-85-30-130-74.eu.ngrok.io/";

lazy_static! {
    static ref MAP: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
}

fn main() {
    // This example demonstrates how to use websockets with rouille.

    // Small message so that people don't need to read the source code.
    // Note that like all examples we only listen on `localhost`, so you can't access this server
    // from another machine than your own.
    println!("Now listening on localhost:8000");

    rouille::start_server("localhost:8000", move |request| {
        router!(request,
            (GET) (/) => {
                // The / route outputs an HTML client so that the user can try the websockets.
                // Note that in a real website you should probably use some templating system, or
                // at least load the HTML from a file.
                Response::html("<script type=\"text/javascript\">
                    var socket = new WebSocket(\"ws://localhost:8000/ws\", \"echo\");
                    function send(data) {{
                        socket.send(data);
                    }}
                    socket.onmessage = function(event) {{
                        document.getElementById('result').innerHTML += event.data + '<br />';
                    }}
                    </script>
                    <h1>UMAIN Days of Exploration & Tech Hackathon</h1>
                    <h2>Team #jumping-raccoons</h2>
                    <p>This example sends back everything you send to the server.</p>
                    <p><form onsubmit=\"send(document.getElementById('msg').value); return false;\">
                    <input type=\"text\" id=\"msg\" />
                    <button type=\"submit\">Send</button>
                    </form></p>
                    <p>Received: </p>
                    <p id=\"result\"></p>")
            },

            (POST) (/) => {
                                println!("URL --> {:?}", request.url());




                let data = try_or_400!(post_input!(request, {
                        // txt: String,
                        // data: String,
                        file: Vec<rouille::input::post::BufferedFile>,
                    }));

                // We just print what was received on stdout. Of course in a real application
                // you probably want to process the data, eg. store it in a database.
                println!("Received data: {:?}", data);
                let reqid = request.header("Requestid").unwrap();



                let d = &data.file.get(0).unwrap().data;
                MAP.lock().unwrap().insert(reqid.to_string(), d.to_vec());

                // let filename = request.get_param("filename");
                // // let requestid = match request.get_param("requestid") {
                // //     Some(x) => { x }
                // //     None => { return Response::text("nope"); }
                // // };
                // let requestid = request.get_param("requestid");
                //
                println!("POST /: {:#?}", request);
                // println!("requestid --> {:?}", requestid);
                // println!("filename --> {:?}", filename);
                //
                // let mut data = request.data().expect("Oops, body already retrieved, problem \
                //                           in the server");
                //
                // let mut buf = Vec::new();
                // match data.read_to_end(&mut buf) {
                //     Ok(n) => {
                //         println!("buf --> {:?}", n);
                //           // let mut out = File::create("image.jpg")?;
                //             // write!(out,n)?;
                //         // MAP.lock().unwrap().insert(requestid, buf);
                //     },
                //     Err(_) => return Response::text("Failed to read body")
                // };



                Response::text("download")
            },
            (GET) (/download) => {
                let start = Instant::now();
                let id = &uuid::Uuid::new_v4().to_string();
                println!("GET /download request id: {}", id);

                // https://hooks.zapier.com/hooks/catch/128523/bvffe6n/?filename=edde&requestid=0185eede&callback=https%3A%2F%2F2804-85-30-130-74.eu.ngrok.io
                let client = reqwest::blocking::Client::new();
                let req = client.get(ZAPIER_URL)
                    .query(&[
                    ("requestid", id),
                    ("filename", &request.get_param("name").unwrap()),
                    ("callback", &String::from(CALLBACK_URL)),
                ]);

                match req.send() {
                    Err(_) => { println!("fail") }
                    Ok(_) => { println!("success") }
                };

                loop {
                    let d = start.elapsed();
                    if d.as_secs() > 20 {
                        break Response::text("timed out after 20 secs")
                    }
                    let map = MAP.lock().unwrap();
                    match map.get(id) {
                        None => { continue; }
                        Some(x) => {
                            // break Response::text(format!("GET OK {}", x))
                            break Response::from_data("application/octet-stream", x.deref());
                        }
                    }
                }
            },
            (GET) (/ws) => {
                // This is the websockets route.

                // In order to start using websockets we call `websocket::start`.
                // The function returns an error if the client didn't request websockets, in which
                // case we return an error 400 to the client thanks to the `try_or_400!` macro.
                //
                // The function returns a response to send back as part of the `start_server`
                // function, and a `websocket` variable of type `Receiver<Websocket>`.
                // Once the response has been sent back to the client, the `Receiver` will be
                // filled by rouille with a `Websocket` object representing the websocket.
                let (response, websocket) = try_or_400!(websocket::start(&request, Some("echo")));

                // Because of the nature of I/O in Rust, we need to spawn a separate thread for
                // each websocket.
                thread::spawn(move || {
                    // This line will block until the `response` above has been returned.
                    let ws = websocket.recv().unwrap();
                    // We use a separate function for better readability.
                    websocket_handling_thread(ws);
                });

                response
            },

            // Default 404 route as with all examples.
            _ => rouille::Response::empty_404()
        )
    });
}

// Function run in a separate thread.
fn websocket_handling_thread(mut websocket: websocket::Websocket) {
    // We wait for a new message to come from the websocket.
    while let Some(message) = websocket.next() {
        match message {
            websocket::Message::Text(txt) => {
                // If the message is text, send it back with `send_text`.
                println!("received {:?} from a websocket", txt);
                websocket.send_text(&txt).unwrap();
            }
            websocket::Message::Binary(_) => {
                println!("received binary from a websocket");
            }
        }
    }
}
