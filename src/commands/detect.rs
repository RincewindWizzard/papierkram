use log::debug;
use rusqlite::Connection;
use crate::config::ApplicationConfig;
use crate::datastore::DataStore;


pub fn main(config: ApplicationConfig, mut connection: Connection) {
    use std::process::Command;
    let mut results = Vec::new();


    for (name, probe) in config.probes {
        debug!("Running {name}: {}", probe.command);


        let result = Command::new("sh")
            .arg("-c")
            .arg(probe.command)
            .output()
            .expect("failed to execute process")
            .status
            .success();

        if result {
            results.push(connection.insert_current_event(&name));
            println!("Detected {}", name);
        } else {
            debug!("{name} was not detected.");
        }
    }

    for result in results {
        result.expect("There was an error while saving the result!");
    }
}
