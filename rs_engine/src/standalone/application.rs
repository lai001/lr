#[cfg(feature = "network")]
use crate::network::NetworkModule;
#[cfg(feature = "plugin_shared_crate")]
use crate::plugin::plugin_crate::Plugin;
use crate::{
    content::{content_file_type::EContentFileType, level::Level},
    engine::Engine,
    input_mode::EInputMode,
    player_viewport::PlayerViewport,
    scene_node::ChangedStateFlags,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
#[cfg(feature = "network")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NetworkObjectData {
    pub id: uuid::Uuid,
    pub replicated: Vec<u8>,
    pub call: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_description: Option<String>,
}

#[cfg(feature = "network")]
impl NetworkObjectData {
    pub fn is_valid(&self) -> bool {
        !(self.call.is_empty() && self.replicated.is_empty())
    }
}

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ReplicatedFieldType {
    Level,
    NetworkReplicated,
    Call,
}

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct EndpointData {
    pub network_object_datas: Vec<NetworkObjectData>,
}

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ServerNetData {
    pub endpoint_data: EndpointData,
    pub client_net_datas: Vec<Vec<u8>>,
    pub level_net_data: Vec<u8>,
}

#[cfg(feature = "network")]
impl ServerNetData {
    pub fn is_valid(&self) -> bool {
        !(self.level_net_data.is_empty()
            && self.client_net_datas.is_empty()
            && self.endpoint_data.network_object_datas.is_empty())
    }

    pub fn serialize(&self) -> rs_artifact::error::Result<Vec<u8>> {
        rs_artifact::bincode_legacy::serialize(&self, Some(rs_artifact::EEndianType::Little))
    }

    pub fn deserialize(data: &[u8]) -> rs_artifact::error::Result<ServerNetData> {
        let server_net_data = rs_artifact::bincode_legacy::deserialize::<ServerNetData>(
            data,
            Some(rs_artifact::EEndianType::Little),
        );
        server_net_data
    }

    pub fn client_endpoint_datas(&self) -> Vec<EndpointData> {
        let mut client_endpoint_datas: Vec<EndpointData> = vec![];
        for client_net_data in &self.client_net_datas {
            let Ok(client_endpoint_data) = rs_artifact::bincode_legacy::deserialize::<EndpointData>(
                client_net_data,
                Some(rs_artifact::EEndianType::Little),
            ) else {
                continue;
            };
            client_endpoint_datas.push(client_endpoint_data);
        }
        client_endpoint_datas
    }

    pub fn level(&self) -> Option<Level> {
        let level = rs_artifact::bincode_legacy::deserialize::<Level>(&self.level_net_data, None);
        level.ok()
    }

    pub fn serialize_level(&mut self, level: &Level) -> rs_artifact::error::Result<()> {
        let data =
            rs_artifact::bincode_legacy::serialize(level, Some(rs_artifact::EEndianType::Little))?;
        self.level_net_data = data;
        Ok(())
    }
}

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClientNetData {
    pub endpoint_data: EndpointData,
}

#[cfg(feature = "network")]
impl ClientNetData {
    pub fn is_valid(&self) -> bool {
        !self.endpoint_data.network_object_datas.is_empty()
    }

    pub fn serialize(&self) -> rs_artifact::error::Result<Vec<u8>> {
        rs_artifact::bincode_legacy::serialize(&self, Some(rs_artifact::EEndianType::Little))
    }

    pub fn deserialize(data: &[u8]) -> rs_artifact::error::Result<ClientNetData> {
        let client_net_data = rs_artifact::bincode_legacy::deserialize::<ClientNetData>(
            data,
            Some(rs_artifact::EEndianType::Little),
        );
        client_net_data
    }
}

#[cfg(feature = "network")]
pub struct NetModule {
    pub is_authority: bool,
    pub server: Option<rs_network::server::Server>,
    pub client: Option<rs_network::client::Client>,
    pub connections: Vec<rs_network::server::Connection>,
}

#[cfg(feature = "network")]
impl NetModule {
    pub fn new() -> NetModule {
        NetModule {
            is_authority: false,
            server: None,
            client: None,
            connections: vec![],
        }
    }
}

#[cfg(feature = "network")]
enum ClientTickResultType {
    OpenLevel(Level),
    None,
}

pub struct Application {
    _window_id: isize,
    player_view_port: PlayerViewport,
    current_active_level: SingleThreadMutType<Level>,
    _contents: Vec<EContentFileType>,
    #[cfg(feature = "plugin_shared_crate")]
    plugins: SingleThreadMutType<Vec<Box<dyn Plugin>>>,
    #[cfg(feature = "network")]
    pub net_module: NetModule,
    pub frame: u32,
    #[cfg(feature = "network")]
    server_pending_open_level: Option<url::Url>,
}

impl Application {
    pub fn new(
        window_id: isize,
        width: u32,
        height: u32,
        engine: &mut Engine,
        current_active_level: &Level,
        contents: Vec<EContentFileType>,
        input_mode: EInputMode,
        #[cfg(feature = "plugin_shared_crate")] mut plugins: Vec<Box<dyn Plugin>>,
    ) -> Application {
        // let resource_manager = ResourceManager::default();

        // let global_sampler_handle = resource_manager.next_sampler();
        // let command = RenderCommand::CreateSampler(CreateSampler {
        //     handle: *global_sampler_handle,
        //     sampler_descriptor: wgpu::SamplerDescriptor::default(),
        // });
        // engine.send_render_command(command);

        // let infos = engine.get_virtual_texture_source_infos();
        let mut player_view_port = PlayerViewport::from_window_surface(
            window_id, width, height, // global_sampler_handle,
            engine, // infos,
            input_mode, false,
        );
        let mut current_active_level =
            current_active_level.make_copy_for_standalone(engine, &contents, &mut player_view_port);

        current_active_level.initialize(engine, &contents, &mut player_view_port);
        current_active_level.set_physics_simulate(true);

        #[cfg(feature = "plugin_shared_crate")]
        for plugin in plugins.iter_mut() {
            plugin.on_init(
                engine,
                &mut current_active_level,
                &mut player_view_port,
                &contents,
            );
        }

        #[cfg(feature = "plugin_shared_crate")]
        for plugin in plugins.iter_mut() {
            plugin.on_open_level(
                engine,
                &mut current_active_level,
                &mut player_view_port,
                &contents,
            );
        }

        Application {
            _window_id: window_id,
            player_view_port,
            #[cfg(feature = "plugin_shared_crate")]
            plugins: SingleThreadMut::new(plugins),
            current_active_level: SingleThreadMut::new(current_active_level),
            _contents: contents,
            #[cfg(feature = "network")]
            net_module: NetModule::new(),
            frame: 0,
            #[cfg(feature = "network")]
            server_pending_open_level: None,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent) {
        self.player_view_port.on_device_event(device_event);
        #[cfg(feature = "plugin_shared_crate")]
        {
            let mut plugins = self.plugins.borrow_mut();
            for plugin in plugins.iter_mut() {
                plugin.on_device_event(device_event);
            }
        }
    }

    pub fn on_window_input(
        &mut self,
        #[cfg(not(target_os = "android"))] window: &mut winit::window::Window,
        ctx: &egui::Context,
        ty: crate::input_type::EInputType,
    ) -> Vec<winit::keyboard::KeyCode> {
        #[cfg(not(target_os = "android"))]
        let _ = window;
        self.player_view_port.on_window_input(ty.clone());
        #[cfg(feature = "plugin_shared_crate")]
        let mut consume = vec![];
        #[cfg(not(feature = "plugin_shared_crate"))]
        let consume = vec![];
        #[cfg(feature = "plugin_shared_crate")]
        {
            let mut plugins = self.plugins.borrow_mut();
            for plugin in plugins.iter_mut() {
                #[cfg(not(target_os = "android"))]
                let mut plugin_consume = plugin.on_window_input(window, ctx.clone(), ty.clone());
                #[cfg(target_os = "android")]
                let mut plugin_consume = plugin.on_window_input(ctx.clone(), ty.clone());
                consume.append(&mut plugin_consume);
            }
        }
        consume
    }

    pub fn on_redraw_requested(
        &mut self,
        engine: &mut Engine,
        ctx: egui::Context,
        #[cfg(not(target_os = "android"))] window: &mut winit::window::Window,
        #[cfg(not(target_os = "android"))] virtual_key_code_states: &std::collections::HashMap<
            winit::keyboard::KeyCode,
            winit::event::ElementState,
        >,
    ) {
        let _ = ctx;
        #[cfg(not(target_os = "android"))]
        let _ = window;

        #[cfg(feature = "network")]
        self.net_tick(engine);

        #[cfg(not(target_os = "android"))]
        self.player_view_port
            .on_window_input(crate::input_type::EInputType::KeyboardInput(
                virtual_key_code_states,
            ));

        let active_level = self.current_active_level.clone();
        {
            let mut active_level = active_level.borrow_mut();

            for actor in active_level.actors.clone() {
                let actor = actor.borrow();
                let mut changed_state = ChangedStateFlags::empty();
                for (_, scene_node) in actor.collect_node_map() {
                    let scene_node = scene_node.borrow();
                    if let Some(state) = scene_node.changed_state() {
                        changed_state.insert(state);
                    }
                }
                if changed_state.contains(ChangedStateFlags::Transformation) {
                    actor
                        .scene_node
                        .borrow_mut()
                        .notify_transformation_updated(active_level.get_physics_mut());
                }
            }
        }

        #[cfg(feature = "plugin_shared_crate")]
        {
            let plugins = self.plugins.clone();
            let mut plugins = plugins.borrow_mut();
            #[cfg(feature = "network")]
            if self.server_pending_open_level.take().is_some() {
                self.notify_level_opend(engine, plugins.iter_mut());
            }
            for plugin in plugins.iter_mut() {
                #[cfg(not(target_os = "android"))]
                plugin.tick(engine, ctx.clone(), &self._contents.clone(), self, window);
                #[cfg(target_os = "android")]
                plugin.tick(engine, ctx.clone(), &self._contents.clone(), self);
            }
        }

        let mut active_level = self.current_active_level.borrow_mut();

        self.player_view_port.update_global_constants(engine);

        if let Some(physics) = active_level.get_physics_mut() {
            physics.collision_events.clear();
        }
        active_level.tick(engine.get_game_time(), engine, &mut self.player_view_port);
        let mut draw_objects = active_level.collect_draw_objects();
        for draw_object in draw_objects.iter_mut() {
            self.player_view_port
                .update_draw_object(engine, draw_object);
            draw_object.switch_player_viewport(&self.player_view_port);
        }
        self.player_view_port.append_to_draw_list(&draw_objects);

        if let Some(physics) = active_level.get_physics_mut() {
            self.player_view_port.physics_debug(
                engine,
                &physics.rigid_body_set,
                &physics.collider_set,
            );
        }
        engine.present_player_viewport(&mut self.player_view_port);

        self.frame += 1;
    }

    pub fn on_size_changed(&mut self, engine: &mut Engine, width: u32, height: u32) {
        self.player_view_port.size_changed(width, height, engine);
    }

    #[cfg(feature = "plugin_shared_crate")]
    pub fn reload_plugins(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        *self.plugins.borrow_mut() = plugins;
    }

    pub fn current_active_level(&self) -> SingleThreadMutType<Level> {
        self.current_active_level.clone()
    }

    pub fn player_view_port_mut(&mut self) -> &mut PlayerViewport {
        &mut self.player_view_port
    }

    pub fn window_id(&self) -> isize {
        self._window_id
    }
}

#[cfg(feature = "network")]
impl Application {
    pub fn open_server(&mut self, server_addr: std::net::SocketAddr) {
        match rs_network::server::Server::bind(server_addr) {
            Ok(server) => {
                self.net_module.server = Some(server);
                self.net_module.is_authority = true;
            }
            Err(err) => {
                log::error!("{}", err);
            }
        }
    }

    pub fn connect_to_server(&mut self, server_addr: std::net::SocketAddr) {
        match rs_network::client::Client::bind(server_addr, Some(format!(""))) {
            Ok(client) => {
                self.net_module.client = Some(client);
                self.net_module.is_authority = false;
            }
            Err(err) => {
                log::error!("{}", err);
            }
        }
    }

    fn server_tick(
        active_level: &mut Level,
        server: &mut rs_network::server::Server,
        engine: &mut Engine,
        contents: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) -> Vec<rs_network::server::Connection> {
        use std::collections::HashMap;

        let mut endpoint_data: EndpointData = EndpointData::default();
        let new_connections = server.process_incoming();
        {
            active_level.visit_network_replicated_mut(&mut |network_replicated| {
                let network_object_data = NetworkObjectData {
                    id: *network_replicated.get_network_id(),
                    replicated: network_replicated.on_replicated(),
                    call: network_replicated.call(),
                    debug_description: network_replicated.debug_name(),
                };
                if network_object_data.is_valid() {
                    endpoint_data.network_object_datas.push(network_object_data);
                }
            });
        }

        let mut peer_addr_to_client: HashMap<
            std::net::SocketAddr,
            &mut rs_network::client::Client,
        > = HashMap::new();
        let mut peer_addr_to_messages: HashMap<
            std::net::SocketAddr,
            Vec<rs_network::codec::Message>,
        > = HashMap::new();
        let mut peer_addr_to_send: HashMap<std::net::SocketAddr, Vec<u8>> = HashMap::new();

        for client in server.clients_mut() {
            let mut messages = client.take_messages();
            messages.retain(|x| !x.data.is_empty());
            peer_addr_to_messages.insert(client.peer_addr, messages);
            peer_addr_to_client.insert(client.peer_addr, client);
        }

        for peer_addr in peer_addr_to_client.keys() {
            let mut messages: Vec<&Vec<rs_network::codec::Message>> = vec![];
            let mut client_net_datas: Vec<Vec<u8>> = vec![];
            for pair in &peer_addr_to_messages {
                if pair.0 != peer_addr {
                    messages.push(pair.1);
                }
            }
            let messages = messages.into_iter().flatten();
            for message in messages {
                client_net_datas.push(message.data.clone());
            }
            let server_net_data = ServerNetData {
                endpoint_data: endpoint_data.clone(),
                client_net_datas,
                level_net_data: vec![],
            };
            if server_net_data.is_valid() {
                match server_net_data.serialize() {
                    Ok(data) => {
                        peer_addr_to_send.insert(*peer_addr, data);
                    }
                    Err(err) => {
                        log::warn!("{}", err)
                    }
                }
            }
        }

        for (peer_addr, data) in peer_addr_to_send {
            let client = peer_addr_to_client.get_mut(&peer_addr).unwrap();
            client.write(data);
        }

        for messages in peer_addr_to_messages.values() {
            for message in messages {
                debug_assert!(!message.data.is_empty());
                let Ok(client_net_data) = ClientNetData::deserialize(&message.data) else {
                    continue;
                };
                let mut sync_ids = vec![];
                for network_object_data in &client_net_data.endpoint_data.network_object_datas {
                    active_level.visit_network_replicated_mut(&mut |network_replicated| {
                        let id = network_replicated.get_network_id();
                        if id == &network_object_data.id {
                            #[cfg(feature = "network_debug_trace")]
                            log::trace!(
                                "[Server]On sync, id: {id}, name: {:?}",
                                network_replicated.debug_name()
                            );
                            sync_ids.push(id.clone());
                            network_replicated.on_sync(&network_object_data.replicated);
                            network_replicated.on_call(&network_object_data.call);
                            network_replicated.on_sync2(
                                &network_object_data.replicated,
                                &network_object_data.call,
                                engine,
                                contents,
                                player_viewport,
                            );
                        }
                    });
                }
                if sync_ids.len() != client_net_data.endpoint_data.network_object_datas.len() {
                    log::warn!("[Server]Some objects are not synchronized");
                    for network_object_data in &client_net_data.endpoint_data.network_object_datas {
                        if !sync_ids.contains(&network_object_data.id) {
                            let id = &network_object_data.id;
                            match &network_object_data.debug_description {
                                Some(debug_description) => {
                                    log::warn!(
                                        "[Server]{id} not synchronized, {debug_description}"
                                    );
                                }
                                None => {
                                    log::warn!("[Server]{id} not synchronized");
                                }
                            }
                        }
                    }
                }
            }
        }

        new_connections
    }

    fn client_tick(
        engine: &mut Engine,
        current_active_level: &mut SingleThreadMutType<Level>,
        contents: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
        client: &mut rs_network::client::Client,
    ) -> ClientTickResultType {
        let mut result = ClientTickResultType::None;
        let mut endpoint_data: EndpointData = EndpointData::default();
        {
            let mut active_level = current_active_level.borrow_mut();
            active_level.visit_network_replicated_mut(&mut |network_replicated| {
                let network_object_data = NetworkObjectData {
                    id: *network_replicated.get_network_id(),
                    replicated: network_replicated.on_replicated(),
                    call: network_replicated.call(),
                    debug_description: network_replicated.debug_name(),
                };
                if network_object_data.is_valid() {
                    endpoint_data.network_object_datas.push(network_object_data);
                }
            });
        }

        for message in client.take_messages() {
            if message.data.is_empty() {
                continue;
            }
            let Ok(server_net_data) = ServerNetData::deserialize(&message.data) else {
                continue;
            };
            if let Some(mut remote_level) = server_net_data.level() {
                log::trace!("To remote level: {}", &remote_level.get_name());
                remote_level.initialize(engine, contents, player_viewport);
                result = ClientTickResultType::OpenLevel(remote_level);
            }

            let client_endpoint_datas = server_net_data.client_endpoint_datas();
            let endpoint_data = &server_net_data.endpoint_data;

            let mut active_level = current_active_level.borrow_mut();
            let mut sync_ids = vec![];
            let network_object_datas = std::iter::once(endpoint_data)
                .chain(client_endpoint_datas.iter())
                .flat_map(|ed| ed.network_object_datas.iter())
                .collect::<Vec<&NetworkObjectData>>();
            for network_object_data in &network_object_datas {
                active_level.visit_network_replicated_mut(&mut |network_replicated| {
                    let id = network_replicated.get_network_id();
                    if id == &network_object_data.id {
                        #[cfg(feature = "network_debug_trace")]
                        log::trace!(
                            "[Client]On sync, id: {id}, name: {:?}",
                            network_replicated.debug_name()
                        );
                        sync_ids.push(id.clone());
                        network_replicated.on_sync(&network_object_data.replicated);
                        network_replicated.on_call(&network_object_data.call);
                        network_replicated.on_sync2(
                            &network_object_data.replicated,
                            &network_object_data.call,
                            engine,
                            contents,
                            player_viewport,
                        );
                    }
                });
            }
            if sync_ids.len() != network_object_datas.len() {
                log::warn!("[Client]Some objects are not synchronized");
                for network_object_data in &network_object_datas {
                    if !sync_ids.contains(&network_object_data.id) {
                        let id = &network_object_data.id;
                        match &network_object_data.debug_description {
                            Some(debug_description) => {
                                log::warn!("[Client]{id} not synchronized, {debug_description}");
                            }
                            None => {
                                log::warn!("[Client]{id} not synchronized");
                            }
                        }
                    }
                }
            }
        }

        let client_net_data = ClientNetData { endpoint_data };
        if client_net_data.is_valid() {
            match client_net_data.serialize() {
                Ok(data) => {
                    client.write(data);
                }
                Err(err) => {
                    log::warn!("{}", err)
                }
            }
        }
        result
    }

    fn net_tick(&mut self, engine: &mut Engine) {
        if self.net_module.is_authority {
            if let Some(server) = &mut self.net_module.server {
                let mut active_level = self.current_active_level.borrow_mut();
                let mut new_connections = Application::server_tick(
                    &mut active_level,
                    server,
                    engine,
                    &self._contents,
                    &mut self.player_view_port,
                );
                #[cfg(feature = "plugin_shared_crate")]
                if !new_connections.is_empty() {
                    for plugin in self.plugins.borrow_mut().iter_mut() {
                        plugin.on_new_connections(&new_connections);
                    }
                    self.net_module.connections.append(&mut new_connections);
                }
            }
        } else {
            if let Some(client) = &mut self.net_module.client {
                let result = Application::client_tick(
                    engine,
                    &mut self.current_active_level,
                    &self._contents,
                    &mut self.player_view_port,
                    client,
                );
                match result {
                    ClientTickResultType::OpenLevel(level) => {
                        self.current_active_level = SingleThreadMut::new(level);
                        let plugins = self.plugins.clone();
                        let mut plugins = plugins.borrow_mut();
                        self.on_network_changed();
                        self.notify_level_opend(engine, plugins.iter_mut());
                    }
                    ClientTickResultType::None => {}
                }
            }
        }
    }

    pub fn on_network_changed(&mut self) {
        debug_assert_eq!(
            self.net_module.server.is_some() && self.net_module.client.is_some(),
            false
        );
        if self.net_module.server.is_some() {
            let mut level = self.current_active_level.borrow_mut();
            level.visit_network_replicated_mut(&mut |rep| {
                rep.on_net_mode_changed(crate::network::ENetMode::Server);
            });
            level.network_fields.is_server = true;
        } else if self.net_module.client.is_some() {
            let mut level = self.current_active_level.borrow_mut();
            level.visit_network_replicated_mut(&mut |rep| {
                rep.on_net_mode_changed(crate::network::ENetMode::Client);
            });
            level.network_fields.is_server = false;
        }
    }

    pub fn open_server_level(&mut self, engine: &mut Engine, url: url::Url) -> Option<()> {
        let mut find_level = self
            ._contents
            .iter()
            .find(|x| match x {
                EContentFileType::Level(level) => level.borrow().url == url,
                _ => false,
            })
            .and_then(|x| match x {
                EContentFileType::Level(level) => Some(level.borrow().make_copy_for_standalone(
                    engine,
                    &self._contents,
                    &mut self.player_view_port,
                )),
                _ => None,
            })?;

        if !self.net_module.is_authority {
            return None;
        }
        let Some(server) = &mut self.net_module.server else {
            return None;
        };
        find_level.initialize(engine, &self._contents, &mut self.player_view_port);
        find_level.set_physics_simulate(true);
        let mut server_net_data = ServerNetData {
            endpoint_data: EndpointData::default(),
            client_net_datas: vec![],
            level_net_data: vec![],
        };
        server_net_data.serialize_level(&find_level).ok()?;
        let data = server_net_data.serialize().ok()?;
        server.broadcast(&data);
        self.current_active_level = SingleThreadMut::new(find_level);
        self.server_pending_open_level = Some(url);
        self.on_network_changed();
        return Some(());
    }

    fn notify_level_opend<'a>(
        &mut self,
        engine: &mut Engine,
        plugins: impl Iterator<Item = &'a mut Box<dyn Plugin>>,
    ) {
        for plugin in plugins {
            plugin.on_open_level(
                engine,
                &mut self.current_active_level.borrow_mut(),
                &mut self.player_view_port,
                &self._contents,
            );
        }
    }
}
