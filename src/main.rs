use std::collections::{HashMap, HashSet};
use std::{fs, thread};
use std::fs::File;
use crossbeam_channel::{unbounded, Receiver, Sender};
use wg_2024::config::Config;
use wg_2024::drone::Drone;
use dronegowski::Dronegowski;
use log::LevelFilter;
use simplelog::{ConfigBuilder, WriteLogger};
use SimulationController::DronegowskiSimulationController;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

/*struct SimulationController {
    nodes_channels: HashMap<NodeId, Sender<DroneCommand>>,
    sim_controller_event_recv: Receiver<DroneEvent>,
}

impl SimulationController {
    fn new(
        nodes_channels: HashMap<NodeId, Sender<DroneCommand>>,
        sim_controller_event_recv: Receiver<DroneEvent>,
    ) -> Self {
        Self {
            nodes_channels,
            sim_controller_event_recv
        }
    }

    fn crash_all(&mut self) {
        for (_, sender) in self.nodes_channels.iter() {
            sender.send(DroneCommand::Crash).unwrap();
        }
    }
}*/

fn main(){

    // Logger di simplelog
    let log_level = LevelFilter::Info;
    let _logger = WriteLogger::init(
        log_level,
        ConfigBuilder::new().set_thread_level(log_level).build(),
        File::create("output.log").expect("Could not create log file"),
    );

    let config = parse_config("config_file/config.toml");
    parse_node(config);
}

// config.toml -> Config
pub fn parse_config(file: &str) -> Config {
    let file_str = fs::read_to_string(file).expect("error reading config file");
    println!("Parsing configuration file...");
    toml::from_str(&file_str).expect("Error occurred during config file parsing")
}

fn parse_node(config: Config) {
    let (sim_event_send, sim_event_recv) = crossbeam_channel::unbounded::<DroneEvent>();

    let mut channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)> = HashMap::new();
    let mut sim_command_channels: HashMap<NodeId, Sender<DroneCommand>> = HashMap::new();

    for drone in &config.drone {
        let (packet_send, packet_recv) = unbounded();
        channels.insert(drone.id, (packet_send, packet_recv));
    }
    for client in &config.client {
        let (packet_send, packet_recv) = unbounded();
        channels.insert(client.id, (packet_send, packet_recv));
    }
    for server in &config.server {
        let (packet_send, packet_recv) = unbounded();
        channels.insert(server.id, (packet_send, packet_recv));
    }

    let mut handles = Vec::new();

    // Creazione dei droni
    for drone in config.drone.clone().into_iter() {
        let packet_recv = channels[&drone.id].1.clone(); // Packet Receiver Drone (canale su cui riceve i pacchetti il drone)
        let node_event_send = sim_event_send.clone(); // Controller Send Drone (canale del SC su cui può inviare gli eventi il drone)
        let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new(); // Packet Send Drone (canali dei nodi vicini a cui può inviare i pacchetti il drone)

        for neighbour_id in drone.connected_node_ids.clone() {
            let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
            neighbours.insert(neighbour_id, channel_neighbour.0.clone());
        }

        let (command_send, command_recv) = unbounded();
        sim_command_channels.insert(drone.id, command_send);

        handles.push(thread::spawn(move || {
            let mut drone = Dronegowski::new(drone.id, node_event_send, command_recv, packet_recv, neighbours, drone.pdr);

            drone.run();
        }));
    }

    // Creazione dei client
    for client in &config.client {

        // let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new();
        // for neighbour_id in client.connected_drone_ids.clone() {
        //     let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
        //     neighbours.insert(neighbour_id, channel_neighbour.0.clone());
        // }

        //let (command_send, command_recv) = unbounded();
        // sim_command_channels.insert(client.id, command_send);

        // handles.push(thread::spawn(move || {
        //      let mut client = Client::new(...);
        //
        //      client.run();
        // }));
    }

    // Creazione dei server
    for server in &config.server {

        // let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new();
        // for neighbour_id in server.connected_drone_ids.clone() {
        //     let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
        //     neighbours.insert(neighbour_id, channel_neighbour.0.clone());
        // }

        //let (command_send, command_recv) = unbounded();
        // sim_command_channels.insert(server.id, command_send);

        // handles.push(thread::spawn(move || {
        //      let mut server = Server::new(...);
        //
        //      server.run();
        // }));

    }

    // Passa la lista di nodi al SimulationController
        DronegowskiSimulationController::new(config, sim_command_channels, sim_event_recv);



    // let simulation_controller = SimulationController::

    // Prova di crash
    //simulation_controller.crash_all();

    while let Some(handle) = handles.pop() {
        handle.join().unwrap();
    }

}
