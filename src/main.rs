use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::Receiver;
use wg_2024::config::Config;
use wg_2024::drone::Drone;
use dronegowski::Dronegowski;
use simulationcontroller;

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
    node_vec: HashSet<T>,
    sim_controller_event_recv: Receiver<DroneEvent>,
    sim_controller_command_send: Sender<DroneCommand>,
}

fn main(){
    let config = parse_file("tests/common/config.toml");

    //let drone = Dronegowski::new();
}

fn parse_file(file: &str) -> Config {
    let file_config =fs::read_to_string(file).expect("error reading config file");
    println!("Parsing configuration file...");
    toml::from_str(&file_str).expect("Error occurred during config file parsing")
}

fn parse_node(config: Config){
    let (sim_event_send, sim_event_recv) = unbuonded();
    let mut channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)> = HashMap::new();
    let mut sim_command_channels: HashMap<NodeId, Sender<DroneCommand>> = HashMap::new();

    for node in config{
        let (packet_send, packet_recv) = unbounded();
        channels.insert(node.id, (packet_send, packet_recv));
    }

    for node in config{
        let mut packet_send: HashMap<NodeId, Sender<Packet>> = HashMap::new();
        for neighbour_id in node.connected_node_ids{
            let Some(channel_neighbour) = channels.get(neighbour_id);
            packet_send.insert(neighbour_id, channel_neighbour.0);
        }

        let (sim_command_send, sim_command_recv) = unbuonded();
        sim_command_channels.insert(node.id, sim_command_send);

        let Some(command_channel) = sim_command_channels.get(node_id);
        let Some(channel) = channels.get(node.id);

        if node.drone{
            let drone = Dronegowski::new(
                node.id,
                sim_event_send,
                sim_command_recv,
                channel.1,
                packet_send,
                node.pdr
            );
        }

        else if node.client{
            let client = Client::new(
                node.id,
                sim_event_send,
                sim_command_recv,
                channel.1,
                packet_send,
            );
        }

        else if node.server{
            let server = Server::new(
                node.id,
                sim_event_send,
                sim_command_recv,
                channel.1,
                packet_send,
            );
        }

        Self.node_vec.insert()
    }
    let simulation_controller = SimulationController::create(node_vec, sim_command_channels, sim_event_recv);
}
impl client for Client{
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

impl server for Server{
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