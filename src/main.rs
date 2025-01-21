use std::collections::{HashMap, HashSet};
use std::fs;
use crossbeam_channel::{unbounded, Receiver, Sender};
use wg_2024::config::Config;
use wg_2024::drone::Drone;
use dronegowski::Dronegowski;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
// use SimulationController;

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

struct SimulationController{
    node_vec: Vec<Node>,
    sim_controller_event_recv: Receiver<DroneEvent>,
    sim_controller_command_send: HashMap<NodeId, Sender<DroneCommand>>,
}

struct Node {
    node_id: u8,
}

impl Node {
    fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
        }
    }
}

impl SimulationController {
    fn new(node_vec: Vec<Node>, sim_controller_command_send: HashMap<NodeId, Sender<DroneCommand>>, sim_controller_event_recv: Receiver<DroneEvent> ) -> Self {
        Self {
            node_vec,
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

fn parse_node(config: Config){
    let (sim_event_send, sim_event_recv) = unbounded();
    let mut channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)> = HashMap::new();
    let mut sim_command_channels: HashMap<NodeId, Sender<DroneCommand>> = HashMap::new();

    let mut node_vec = Vec::new();

    for drone in &config.drone {
        channels.insert(drone.id, unbounded());
    }
    for client in &config.client {
        channels.insert(client.id, unbounded());
    }
    for server in &config.server {
        channels.insert(server.id, unbounded());
    }

    for node in config.drone{

        let mut packet_send: HashMap<NodeId, Sender<Packet>> = HashMap::new();
        for neighbour_id in &node.connected_node_ids{
            let Some(channel_neighbour) = channels.get(&neighbour_id) else {
                panic!("Channel for neighbour_id {} not found", neighbour_id);
            };
            packet_send.insert(*neighbour_id, channel_neighbour.clone().0);

            packet_send.insert(*neighbour_id, channel_neighbour.clone().0);
        }

        let (sim_command_send, sim_command_recv) = unbounded();
        sim_command_channels.insert(node.id, sim_command_send);

        let Some(command_channel) = sim_command_channels.get(&node.id) else {
            panic!("Command channel for node.id {} not found", node.id);
        };

        let Some(channel) = channels.get(&node.id) else {
            panic!("Channel for node.id {} not found", node.id);
        };

        let drone = Dronegowski::new(
            node.id,
            sim_event_send.clone(),
            sim_command_recv,
            channel.clone().1,
            packet_send,
            node.pdr
        );

        node_vec.push(Node { node_id: node.id });
    }

    for node in config.server{

        let mut packet_send: HashMap<NodeId, Sender<Packet>> = HashMap::new();
        for neighbour_id in &node.connected_drone_ids{
            let Some(channel_neighbour) = channels.get(&neighbour_id) else {
                panic!("Channel for node.id {} not found", node.id);
            };
            packet_send.insert(*neighbour_id, channel_neighbour.clone().0);
        }

        let (sim_command_send, sim_command_recv) = unbounded();
        sim_command_channels.insert(node.id, sim_command_send);

        let Some(command_channel) = sim_command_channels.get(&node.id) else {
            panic!("Command channel for node.id {} not found", node.id);
        };
        let Some(channel) = channels.get(&node.id) else {
            panic!("Channel for node.id {} not found", node.id);
        };

        let server = Server::new(
            node.id,
            sim_event_send.clone(),
            sim_command_recv,
            channel.clone().1,
            packet_send,
        );

        node_vec.push(Node { node_id: node.id });
    }

    for node in config.client{

        let mut packet_send: HashMap<NodeId, Sender<Packet>> = HashMap::new();
        for neighbour_id in &node.connected_drone_ids{
            let Some(channel_neighbour) = channels.get(&neighbour_id) else {
                panic!("Channel neighbour for node.id {} not found", node.id);
            };
            packet_send.insert(*neighbour_id, channel_neighbour.clone().0);
        }

        let (sim_command_send, sim_command_recv) = unbounded();
        sim_command_channels.insert(node.id, sim_command_send);

        let Some(command_channel) = sim_command_channels.get(&node.id) else {
            panic!("Command channel for node.id {} not found", node.id);
        };
        let Some(channel) = channels.get(&node.id) else {
            panic!("Channel for node.id {} not found", node.id);
        };

        let client = Client::new(
            node.id,
            sim_event_send.clone(),
            sim_command_recv,
            channel.clone().1,
            packet_send,
        );

        node_vec.push(Node { node_id: node.id });
    }

    let simulation_controller = SimulationController::new(node_vec, sim_command_channels, sim_event_recv);
}
impl Client{
    fn new(
        id: NodeId,
        sim_controller_send: Sender<DroneEvent>,
        sim_controller_recv: Receiver<DroneCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>
    ) -> Self {
        Self{
            id,
            sim_controller_send,
            sim_controller_recv,
            packet_recv,
            packet_send,
        }
    }
}

impl Server{
    fn new(
        id: NodeId,
        sim_controller_send: Sender<DroneEvent>,
        sim_controller_recv: Receiver<DroneCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>
    ) -> Self {
        Self{
            id,
            sim_controller_send,
            sim_controller_recv,
            packet_recv,
            packet_send,
        }
    }
}