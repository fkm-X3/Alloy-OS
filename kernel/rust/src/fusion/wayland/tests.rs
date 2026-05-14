//! Integration tests for Wayland protocol handlers
//!
//! Tests the complete protocol flow for:
//! - Client connection -> get_registry -> bind compositor -> create surface

#[cfg(test)]
mod wayland_integration_tests {
    use super::super::*;
    use super::super::super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_message_encode_decode_roundtrip() {
        let msg = WaylandMessage {
            object_id: ObjectId::DISPLAY,
            opcode: 0,
            payload: alloc::vec![1, 2, 3, 4],
        };

        let encoded = msg.encode().unwrap();
        assert_eq!(encoded.len(), 12); // 8 header + 4 payload

        let decoded = WaylandMessage::decode(&encoded).unwrap().unwrap();
        assert_eq!(decoded.object_id, ObjectId::DISPLAY);
        assert_eq!(decoded.opcode, 0);
        assert_eq!(decoded.payload, alloc::vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_message_decode_incomplete_header() {
        let buf = [0u8; 4]; // Too short for header
        let result = WaylandMessage::decode(&buf);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_message_decode_invalid_length() {
        // Header claims length > 4096
        let mut buf = [0u8; 16];
        buf[6] = 0x11; // length high byte > 4096
        buf[7] = 0x00;
        let result = WaylandMessage::decode(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_identify_interface_boundaries() {
        assert_eq!(identify_interface(1), InterfaceId::Display);
        assert_eq!(identify_interface(2), InterfaceId::Registry);
        assert_eq!(identify_interface(3), InterfaceId::Compositor);
        assert_eq!(identify_interface(4), InterfaceId::Surface);
        assert_eq!(identify_interface(100), InterfaceId::Surface);
        assert_eq!(identify_interface(101), InterfaceId::Unknown);
        assert_eq!(identify_interface(0), InterfaceId::Unknown);
    }

    #[test]
    fn test_display_handler_sync() {
        let mut handler = DisplayHandler::new();
        let client_id = ClientId(1);
        let mut payload = Vec::new();
        payload.extend_from_slice(&42u32.to_le_bytes());

        let response = handler.handle_request(client_id, 0, &payload).unwrap();
        match response {
            DisplayResponse::SyncAck { callback_id, .. } => {
                assert_eq!(callback_id, 42);
            }
            _ => panic!("Expected SyncAck"),
        }
    }

    #[test]
    fn test_display_handler_get_registry() {
        let mut handler = DisplayHandler::new();
        let client_id = ClientId(1);
        let mut payload = Vec::new();
        payload.extend_from_slice(&2u32.to_le_bytes());

        let response = handler.handle_request(client_id, 1, &payload).unwrap();
        match response {
            DisplayResponse::RegistryCreated { registry_id } => {
                assert_eq!(registry_id, 2);
            }
            _ => panic!("Expected RegistryCreated"),
        }
    }

    #[test]
    fn test_display_handler_invalid_opcode() {
        let mut handler = DisplayHandler::new();
        let client_id = ClientId(1);
        let payload = Vec::new();

        let result = handler.handle_request(client_id, 99, &payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_handler_bind() {
        let mut handler = RegistryHandler::new();
        let client_id = ClientId(1);

        // Build bind payload: name(4) + interface_str + null + version(4) + id(4)
        let mut payload = Vec::new();
        payload.extend_from_slice(&0u32.to_le_bytes()); // global name 0 = wl_compositor
        payload.extend_from_slice(b"wl_compositor\0"); // interface string
        payload.extend_from_slice(&5u32.to_le_bytes()); // version
        payload.extend_from_slice(&3u32.to_le_bytes()); // object id

        let response = handler.handle_request(client_id, 0, &payload).unwrap();
        match response {
            RegistryResponse::Bound { global_name, object_id, interface, version } => {
                assert_eq!(global_name, 0);
                assert_eq!(object_id, 3);
                assert_eq!(version, 5);
            }
        }
    }

    #[test]
    fn test_registry_handler_get_globals() {
        let handler = RegistryHandler::new();
        let events = handler.get_global_events(2);
        assert!(!events.is_empty());
    }

    #[test]
    fn test_compositor_create_surface() {
        let mut handler = CompositorHandler::new();
        let client_id = ClientId(1);

        let mut payload = Vec::new();
        payload.extend_from_slice(&3u32.to_le_bytes()); // object id

        let response = handler.handle_compositor_request(client_id, 0, &payload).unwrap();
        match response {
            CompositorResponse::SurfaceCreated { surface_id, object_id } => {
                assert_eq!(surface_id.0, 1);
                assert_eq!(object_id, 3);
            }
        }
        assert_eq!(handler.surface_count(), 1);
    }

    #[test]
    fn test_compositor_surface_lifecycle() {
        let mut handler = CompositorHandler::new();
        let client_id = ClientId(1);

        // Create surface
        let mut create_payload = Vec::new();
        create_payload.extend_from_slice(&3u32.to_le_bytes());
        handler.handle_compositor_request(client_id, 0, &create_payload).unwrap();

        // Damage surface
        let mut damage_payload = Vec::new();
        damage_payload.extend_from_slice(&0i32.to_le_bytes());
        damage_payload.extend_from_slice(&0i32.to_le_bytes());
        damage_payload.extend_from_slice(&100i32.to_le_bytes());
        damage_payload.extend_from_slice(&100i32.to_le_bytes());
        handler.handle_surface_request(3, 0, &damage_payload).unwrap();

        // Attach buffer
        let mut attach_payload = Vec::new();
        attach_payload.extend_from_slice(&1u32.to_le_bytes()); // buffer id
        attach_payload.extend_from_slice(&0i32.to_le_bytes()); // x
        attach_payload.extend_from_slice(&0i32.to_le_bytes()); // y
        let response = handler.handle_surface_request(3, 1, &attach_payload).unwrap();
        assert_eq!(response, SurfaceResponse::BufferAttached);

        // Commit
        let response = handler.handle_surface_request(3, 2, &[]).unwrap();
        assert_eq!(response, SurfaceResponse::Committed);

        // Destroy
        let response = handler.handle_surface_request(3, 3, &[]).unwrap();
        assert_eq!(response, SurfaceResponse::Destroyed);
        assert_eq!(handler.surface_count(), 0);
    }

    #[test]
    fn test_protocol_handler_display_routing() {
        let mut handler = ProtocolHandler::new();
        handler.initialize();

        let mut display = DisplayHandler::new();
        let mut registry = RegistryHandler::new();
        let mut compositor = CompositorHandler::new();
        let mut buffer_handler = ShmBufferHandler::new();

        // Build a sync message for object_id 1 (wl_display)
        let mut payload = Vec::new();
        payload.extend_from_slice(&1u32.to_le_bytes()); // callback id

        let msg = WaylandMessage {
            object_id: ObjectId::DISPLAY,
            opcode: 0,
            payload,
        };

        let result = handler.handle_message(
            ClientId(1),
            msg,
            &mut display,
            &mut registry,
            &mut compositor,
            &mut buffer_handler,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_protocol_handler_compositor_routing() {
        let mut handler = ProtocolHandler::new();
        handler.initialize();

        let mut display = DisplayHandler::new();
        let mut registry = RegistryHandler::new();
        let mut compositor = CompositorHandler::new();
        let mut buffer_handler = ShmBufferHandler::new();

        // Build a create_surface message for object_id 3 (wl_compositor)
        let mut payload = Vec::new();
        payload.extend_from_slice(&4u32.to_le_bytes()); // new surface id

        let msg = WaylandMessage {
            object_id: ObjectId(3),
            opcode: 0,
            payload,
        };

        let result = handler.handle_message(
            ClientId(1),
            msg,
            &mut display,
            &mut registry,
            &mut compositor,
            &mut buffer_handler,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_surface_state_commit() {
        let mut surface = SurfaceState::new(SurfaceId(1), 3, 1);

        // Attach and damage in pending state
        surface.attach(42, 0, 0);
        surface.damage(DamageRect::new(0, 0, 100, 100));

        assert_eq!(surface.pending.buffer_id, 42);
        assert!(!surface.pending.damage.is_empty());
        assert_eq!(surface.current.buffer_id, 0);

        // Commit
        surface.commit();

        assert_eq!(surface.current.buffer_id, 42);
        assert!(!surface.current.damage.is_empty());
        assert_eq!(surface.pending.buffer_id, 0); // Cleared
    }

    #[test]
    fn test_format_converter_roundtrip() {
        // RGB565 -> XRGB8888 -> RGB565
        let original: u16 = 0xF800; // Red in RGB565
        let xrgb = FormatConverter::rgb565_to_xrgb8888(original);
        let back = FormatConverter::xrgb8888_to_rgb565(xrgb);
        assert_eq!(back, original);
    }

    #[test]
    fn test_alpha_blend_opaque_overwrites() {
        let src = 0xFF123456u32;
        let dst = 0xFFABCDEFu32;
        assert_eq!(FormatConverter::alpha_blend(src, dst), src);
    }

    #[test]
    fn test_alpha_blend_transparent_preserves_dst() {
        let src = 0x00123456u32;
        let dst = 0xFFABCDEFu32;
        assert_eq!(FormatConverter::alpha_blend(src, dst), dst);
    }

    #[test]
    fn test_wayland_error_display() {
        let err = WaylandError::SocketCreationFailed;
        let s = format!("{}", err);
        assert!(s.contains("Socket creation failed"));

        let err = WaylandError::ProtocolViolation;
        let s = format!("{}", err);
        assert!(s.contains("Protocol violation"));
    }

    #[test]
    fn test_seat_manager_lifecycle() {
        let mut manager = SeatManager::new();
        let caps = SeatCapabilities::default_inputs();
        let seat_id = manager.create_seat();

        let binding_id = manager.bind_seat(1, seat_id, caps);
        assert!(manager.get_binding(1, binding_id).is_some());

        manager.remove_client(1);
        assert!(manager.get_binding(1, binding_id).is_none());
    }

    #[test]
    fn test_output_manager_bind_unbind() {
        use crate::graphics::vesa::VesaDisplay;

        let mut manager = OutputManager::new();
        let geom = OutputGeometry::new(0, 0);
        let modes = alloc::vec![OutputMode::new(1024, 768, 60000)];

        let output_id = manager.bind_output(1, geom, modes);
        assert!(manager.get_binding(1, output_id).is_some());

        manager.remove_client(1);
        assert!(manager.get_binding(1, output_id).is_none());
    }

    #[test]
    fn test_wayland_server_client_lifecycle() {
        let mut server = WaylandServer::new();
        assert!(server.client_count() == 0);
        assert!(!server.is_listening());

        // Initialize creates listening socket
        // (skipped in test environment without /run/user/1000)
        // But we can test disconnect on empty
        assert!(server.disconnect_client(ClientId(1)).is_err());
    }

    #[test]
    fn test_frame_timing_behavior() {
        let timing = FrameTiming::new(1000);
        assert_eq!(timing.presented_at, 1000);
        assert!(timing.next_vsync() > 1000);
        assert!(!timing.is_behind(1010));
        assert!(timing.is_behind(2000));
    }

    #[test]
    fn test_compositor_integration_creation() {
        let ci = CompositorIntegration::new();
        assert!(!ci.has_backend());
        assert_eq!(ci.frames_composited(), 0);
        assert!(ci.framebuffer_width() == 0);
        assert!(ci.framebuffer_height() == 0);
    }
}