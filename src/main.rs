use std::collections::{HashMap, HashSet};
use std::fs;
use crossbeam_channel::{unbounded, Receiver, Sender};
use wg_2024::config::Config;
use wg_2024::drone::Drone;
use dronegowski::Dronegowski;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;


struct Client{
    id: NodeId,
    sim_controller_send: Sender<DroneEvent>,                //Channel used to send commands to the SC
    sim_controller_recv: Receiver<DroneCommand>,            //Channel used to receive commands from the SC
    packet_recv: Receiver<Packet>,                          //Channel used to receive packets from nodes
    packet_send: HashMap<NodeId, Sender<Packet>>,           //Map containing the sending channels of neighbour nodes
}

struct Server{
    id: NodeId,
    sim_controller_send: Sender<DroneEvent>,                //Channel used to send commands to the SC
    sim_controller_recv: Receiver<DroneCommand>,            //Channel used to receive commands from the SC
    packet_recv: Receiver<Packet>,                          //Channel used to receive packets from nodes
    packet_send: HashMap<NodeId, Sender<Packet>>,           //Map containing the sending channels of neighbour nodes
}

pub trait Node {
    fn id(&self) -> NodeId;
}

impl Node for Dronegowski {
    fn id(&self) -> NodeId {
        self.clone().get_id()
    }
}

impl Node for Client {
    fn id(&self) -> NodeId {
        self.id
    }
}

impl Node for Server {
    fn id(&self) -> NodeId {
        self.id
    }
}

struct SimulationController {
    nodes: Vec<Box<dyn Node>>, // Collezione di nodi eterogenei
    sim_controller_event_recv: Receiver<DroneEvent>,
    sim_controller_command_send: HashMap<NodeId, Sender<DroneCommand>>,
}

impl SimulationController {
    fn new(
        nodes: Vec<Box<dyn Node>>,
        sim_controller_command_send: HashMap<NodeId, Sender<DroneCommand>>,
        sim_controller_event_recv: Receiver<DroneEvent>,
    ) -> Self {
        Self {
            nodes,
            sim_controller_event_recv,
            sim_controller_command_send,
        }
    }
}

fn main(){
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

    // Collezione eterogenea per tutti i nodi
    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

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

    // Creazione dei droni
    for drone in &config.drone {

        let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new();
        for neighbour_id in drone.connected_node_ids.clone() {
            let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
            neighbours.insert(neighbour_id, channel_neighbour.0.clone());
        }

        let (command_send, command_recv) = unbounded();
        sim_command_channels.insert(drone.id, command_send);

        let drone_instance = Box::new(Dronegowski::new(drone.id, sim_event_send.clone(), command_recv, channels.get(&drone.id).unwrap().1.clone(), neighbours, drone.pdr));

        nodes.push(drone_instance);
    }

    // Creazione dei client
    for client in &config.client {

        let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new();
        for neighbour_id in client.connected_drone_ids.clone() {
            let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
            neighbours.insert(neighbour_id, channel_neighbour.0.clone());
        }

        let (command_send, command_recv) = unbounded();
        sim_command_channels.insert(client.id, command_send);

        let client_instance = Box::new(Client {
            id: client.id,
            sim_controller_send: sim_event_send.clone(),
            sim_controller_recv: command_recv.clone(),
            packet_recv: channels.get(&client.id).unwrap().1.clone(),
            packet_send: neighbours,
        });

        nodes.push(client_instance);
    }

    // Creazione dei server
    for server in &config.server {

        let mut neighbours:HashMap<NodeId, Sender<Packet>> = HashMap::new();
        for neighbour_id in server.connected_drone_ids.clone() {
            let Some(channel_neighbour) = channels.get(&neighbour_id) else { panic!("") };
            neighbours.insert(neighbour_id, channel_neighbour.0.clone());
        }

        let (command_send, command_recv) = unbounded();
        sim_command_channels.insert(server.id, command_send);

        let server_instance = Box::new(Server {
            id: server.id,
            sim_controller_send: sim_event_send.clone(),
            sim_controller_recv: command_recv.clone(),
            packet_recv: channels.get(&server.id).unwrap().1.clone(),
            packet_send: neighbours,
        });

        nodes.push(server_instance);
    }

    // Itera su tutti i nodi
    for node in &nodes {
        println!("Node ID: {:?}", node.id());
    }

    println!("{:?} \n{:?}", sim_command_channels, sim_event_recv);

    // Passa la lista di nodi al SimulationController
    let simulation_controller = SimulationController::new(
        nodes,
        sim_command_channels,
        sim_event_recv,
    );

    // let simulation_controller = SimulationController::


}
