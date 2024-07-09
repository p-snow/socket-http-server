use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::thread;
use std::string::String;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::collections::HashMap;

static DOMAIN_SOCK: &str = "/tmp/local-info.sock";

fn main() -> std::io::Result<()> {
    // remove existing socket up-front
    if fs::metadata(&DOMAIN_SOCK).is_ok() {
        if let Err(_err) = fs::remove_file(&DOMAIN_SOCK) {
            panic!("Error removing file: {}", _err);
        }
    }

    let listener = UnixListener::bind(DOMAIN_SOCK)?;

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // connection succeeded
                println!("Got a client: {:?}", stream);
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                // connection failed
                eprintln!("Error listening: {}", err);
                break;
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: UnixStream) -> std::io::Result<()> {
    // Read data from the client
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).expect("Read error");
    if bytes_read == 0 {
        eprintln!("Client disconnected");
    }
    let received = String::from_utf8_lossy(&buffer[..bytes_read]);

    let message = execute_command(&received);

    stream.write_all(b"HTTP/1.1 200 OK\n")?;
    stream.write_all(b"Content-Type: text/plain\n")?;
    match message {
        Some(s) => {
            stream.write_all(format!("Content-Length: {}\n\n", s.len()).as_bytes())?;
            stream.write_all(format!("{}", s).as_bytes())?;
            Ok(())
        }
        _ => {
            stream.write_all(b"Content-Length: 0\n\n")?;
            eprintln!("HTTP response issued with zero length");
            Ok(())
        }
    }
}

fn execute_command(header: &str) -> Option<String> {
    let mut params: HashMap<&str, &str> = HashMap::new();
    let path: Option<&str> = if let Some(first_line) = header.lines().next() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 && parts.get(0) == Some(&"GET") {
            let query: Vec<&str> = parts[1].split("?").collect();
            for q in query[1].split("&") {
                if let [key, value] = q.split('=').collect::<Vec<&str>>()[..] {
                    params.insert(key, value);
                }
            }
            Some(query[0])
        } else {
            None
        }
    } else {
        None
    };

    match path {
        Some("/info/fs/avail") => {
            if let Some(&f) = params.get("file") {
                let output = Command::new("df")
                    .arg("-h").arg(f)
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to execute command");

                let output2 = Command::new("awk")
                    .arg("NR==2{print $4}")
                    .stdin(Stdio::from(output.stdout.unwrap()))
                    .output()
                    .expect("Failed to execute command");

                if output2.status.success() {
                    println!("Command executed successfully");
                    println!("Output:\n{}", String::from_utf8_lossy(&output2.stdout));
                    return String::from_utf8(output2.stdout).ok();
                } else {
                    eprintln!("Command failed with stderr:\n{}", String::from_utf8_lossy(&output2.stderr));
                }
            } else {
                eprintln!("Key 'file' not found in HashMap");
            }

            return None
        }
        _ => {
            return None
        }
    }
}
