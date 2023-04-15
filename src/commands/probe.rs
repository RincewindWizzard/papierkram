use std::collections::HashMap;
use crate::args::ProbeCommand;
use crate::config::{ApplicationConfig, Probe};

pub fn main(config: &mut ApplicationConfig, sub_command: &ProbeCommand) {
    match sub_command {
        ProbeCommand::Add { event, cmd } => {
            let result = config.add_probe(event.to_string(), cmd.clone());
            if result.is_err() {
                println!("Could not add new probe! {:?}", result);
            } else {
                println!("Probe succesfully added.");
            }
        }
        ProbeCommand::Remove { event } => {
            let result = config.remove_probe(event.to_string());
            if result.is_err() {
                println!("Could not remove probe! {:?}", result);
            } else {
                println!("Probe removed");
            }
        }
        ProbeCommand::Show {} => {
            use serde_derive::{Deserialize, Serialize};
            let local_config = config.clone();

            // exclude all other configurations and show only the probe configuration
            #[derive(Deserialize, Serialize, Debug)]
            struct Probes {
                probes: HashMap<String, Probe>,
            }
            let toml = toml::to_string(&Probes {
                probes: local_config.probes,
            }).expect("Could not serialize to toml");
            println!("{toml}");
        }
    }
}