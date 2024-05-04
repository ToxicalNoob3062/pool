//lets create a simple web server that listens to incoming connections and sends a response
use rahat3062_pool::ThreadPool;
use std::{
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    //get a listener that binds or listens to the address
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    //create a new threadpool instance
    let pool = ThreadPool::new(4, true);

    //loop through the incoming connections
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // //a function pointer will also work instead of closure as it implement all 3 function traits of closure
        // fn job_as_func_pointer() {
        //     handle_connection(stream);
        // }
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

//handle the incoming connections
fn handle_connection(mut stream: TcpStream) {
    //make a buffer to store the incoming data of size 1024bytes filled with 0
    let mut buffer = [0; 1024];
    //store each byte of the incoming data in each index of the buffer
    stream.read(&mut buffer).unwrap();
    //route signatures
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    //check if the incoming data is a get request
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    //read the html file for response contents as a String
    let contents = fs::read_to_string(filename).unwrap();
    //make the response to be sent to the client
    let response = {
        format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        )
    };
    //write the response to the stream as bytes
    stream.write(response.as_bytes()).unwrap();
    //flush the stream to ensure all the data is sent
    stream.flush().unwrap();
}
