use crate::NetworkInitializer;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::thread;
use thiserror::Error;
use wg_2024::config::Config;
use wg_2024::controller::DroneCommand;
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

/// Parsing file  config.toml


impl NetworkInitializer {
    pub fn parse_config(file: &str) -> Config {
        let file_str = fs::read_to_string(file).expect("error reading config file");
        println!("Parsing configuration file...");
        toml::from_str(&file_str).expect("Error occurred during config file parsing")
    }

    pub fn test_initialization() {
        let config = parse_config("config_file/config.toml");

        match validate_config(&config) {
            Ok(_) => println!("Config validation passed!"),
            Err(e) => {
                println!("Config validation failed: {e:?}");
                panic!("Validation failed.");
            }
        }
    }
}
impl Initializer for NetworkInitializer{
    fn run(&self){
        let config = parse_config("config_file/config.toml");

        match validate_config(&config) {
            Ok(_) => println!("Config validation passed!"),
            Err(e) => {
                println!("Config validation failed: {e:?}");
                panic!("Validation failed.");
            }
        }
    }


}