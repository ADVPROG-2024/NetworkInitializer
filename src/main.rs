use std::collections::{HashMap, HashSet};
use std::{fs, thread};
use std::hash::Hash;
use crossbeam_channel::{unbounded, Receiver, Sender};
use wg_2024::config::Config;
use wg_2024::drone::Drone;
use dronegowski::Dronegowski;
use dronegowski_utils::hosts::{ClientCommand, ClientEvent, ClientType, ServerCommand, ServerEvent, ServerType as ST};
use SimulationController::DronegowskiSimulationController;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use client::DronegowskiClient;
use dronegowski_utils::functions::{simple_log, validate_network};
use dronegowski_utils::network::{SimulationControllerNode, SimulationControllerNodeType};
use null_pointer_drone::MyDrone;
use rolling_drone::RollingDrone;
use rust_do_it::RustDoIt;
use rand::Rng;
use servers::{CommunicationServer, ContentServer, DronegowskiServer};
use skylink::SkyLinkDrone;
use bagel_bomber::BagelBomber;
use rustbusters_drone::RustBustersDrone;
use rusty_drones::RustyDrone;
use lockheedrustin_drone::LockheedRustin;
use bobry_w_locie::drone::BoberDrone;

fn main(){
    simple_log();

    let config = parse_config("config_file/config_star.toml");
    parse_node(config);
}

pub fn parse_config(file: &str) -> Config {
    let file_str = fs::read_to_string(file).expect("error reading config file");
    println!("Parsing configuration file...");
    toml::from_str(&file_str).expect("Error occurred during config file parsing")
}

fn parse_node(config: Config) {
    let mut nodi: Vec<SimulationControllerNode> = Vec::new();

    let (sc_drone_event_send, sc_drone_event_recv) = unbounded::<DroneEvent>();
    let (sc_client_event_send, sc_client_event_recv) = unbounded::<ClientEvent>();
    let (sc_server_event_send, sc_server_event_recv) = unbounded::<ServerEvent>();


    let mut channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)> = HashMap::new();
    let mut sc_drone_channels: HashMap<NodeId, Sender<DroneCommand>> = HashMap::new();
    let mut sc_client_channels: HashMap<NodeId, Sender<ClientCommand>> = HashMap::new();
    let mut sc_server_channels: HashMap<NodeId, Sender<ServerCommand>> = HashMap::new();

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
        let drone_event_send = sc_drone_event_send.clone(); // Controller Send Drone (canale del SC su cui può inviare gli eventi il drone)
        let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new(); // Packet Send Drone (canali dei nodi vicini a cui può inviare i pacchetti il drone)
        let mut neighbours_id = Vec::new();

        for neighbour_id in drone.connected_node_ids.clone() {
            let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
            neighbours.insert(neighbour_id, channel_neighbour.0.clone());
            neighbours_id.push(neighbour_id);
        }

        let (command_send, command_recv) = unbounded::<DroneCommand>();
        sc_drone_channels.insert(drone.id, command_send.clone());

        SimulationControllerNode::new(SimulationControllerNodeType::DRONE{ drone_channel: command_send, pdr: drone.pdr }, drone.id, neighbours_id, &mut nodi);

        handles.push(thread::spawn(move || {
            let mut drone = RustDoIt::new(drone.id, drone_event_send, command_recv, packet_recv, neighbours, drone.pdr);

            drone.run();
        }));
    }

    // Creazione dei client
    for client in config.client.clone().into_iter() {
        let packet_recv = channels[&client.id].1.clone(); // Packet Receiver Client (canale su cui riceve i pacchetti il client)
        let client_event_send = sc_client_event_send.clone(); // Controller Send Client (canale del SC su cui può inviare gli eventi il client)
        let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new(); // Packet Send Client (canali dei nodi vicini a cui può inviare i pacchetti il client)
        let mut neighbours_id = Vec::new();

        for neighbour_id in client.connected_drone_ids.clone() {
            let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
            neighbours.insert(neighbour_id, channel_neighbour.0.clone());
            neighbours_id.push(neighbour_id);
        }

        let (command_send, command_recv) = unbounded::<ClientCommand>();
        sc_client_channels.insert(client.id, command_send.clone());

        let client_type = if rand::rngs::ThreadRng::default().random_range(0..=1) == 1 {
            ClientType::ChatClients
        } else {ClientType::WebBrowsers};

        SimulationControllerNode::new(SimulationControllerNodeType::CLIENT{ client_channel: command_send, client_type: client_type.clone()}, client.id, neighbours_id, & mut nodi);

        handles.push(thread::spawn(move || {
            let mut client = DronegowskiClient::new(client.id, client_event_send, command_recv, packet_recv, neighbours, client_type);

            client.run();
        }));
    }

    // Creazione dei server
    for server in config.server.clone().into_iter()  {
        let packet_recv = channels[&server.id].1.clone(); // Packet Receiver Server (canale su cui riceve i pacchetti il server)
        let server_event_send = sc_server_event_send.clone(); // Controller Send Server (canale del SC su cui può inviare gli eventi il server)
        let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new(); // Packet Send Server (canali dei nodi vicini a cui può inviare i pacchetti il server)
        let mut neighbours_id = Vec::new();

        for neighbour_id in server.connected_drone_ids.clone() {
            let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
            neighbours.insert(neighbour_id, channel_neighbour.0.clone());
            neighbours_id.push(neighbour_id);
        }

        let (command_send, command_recv) = unbounded::<ServerCommand>();
        sc_server_channels.insert(server.id, command_send.clone());

        let server_type = if rand::rngs::ThreadRng::default().random_range(0..=1) == 1 {
            ST::Content
        } else {ST::Content};

        match server_type {
            ST::Communication => {
                SimulationControllerNode::new(SimulationControllerNodeType::SERVER{ server_channel: command_send, server_type: server_type.clone() }, server.id, neighbours_id, & mut nodi);

                handles.push(thread::spawn(move || {
                    let mut dronegowski_server = CommunicationServer::new(server.id, server_event_send, command_recv, packet_recv, neighbours, server_type);
                    dronegowski_server.run();
                }));
            },
            ST::Content => {
                SimulationControllerNode::new(SimulationControllerNodeType::SERVER{ server_channel: command_send, server_type: server_type.clone() }, server.id, neighbours_id, & mut nodi);

                handles.push(thread::spawn(move || {
                    let mut dronegowski_server = ContentServer::new(server.id, server_event_send, command_recv, packet_recv, neighbours, server_type, "ContentServerData/file", "ContentServerData/media");
                    dronegowski_server.run();
                }));
            }
        }

    }
    
    validate_network(&nodi).expect("Network non valido!");

    // Passa la lista di nodi al SimulationController
    DronegowskiSimulationController::new(nodi, sc_drone_channels, sc_client_channels, sc_server_channels, sc_drone_event_send, sc_drone_event_recv, sc_client_event_recv, sc_server_event_recv, channels, &mut handles);

    while let Some(handle) = handles.pop() {
        handle.join().unwrap();
    }
}
