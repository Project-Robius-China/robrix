use makepad_widgets::*;
use matrix_sdk::ruma::OwnedRoomId;
use robrix_botfather::{RuntimeKind, SenderSecurityLevel};

use crate::{
    app::AppStateAction,
    botfather::{self, BotfatherAction, commands::help_text},
    login::login_screen::LoginAction,
    logout::logout_confirm_modal::LogoutAction,
    room::BasicRoomDetails,
    sliding_sync::spawn_on_tokio,
    utils::RoomNameId,
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::helpers::*;
    use crate::shared::icon_button::*;
    use crate::shared::styles::*;

    ICON_COLLAPSE = dep("crate://self/resources/icons/triangle_fill.svg")

    SettingsCard = <RoundedView> {
        width: Fill,
        height: Fit,
        flow: Down,
        spacing: 8,
        padding: 14,
        show_bg: true,
        draw_bg: {
            color: (COLOR_BG_PREVIEW)
            border_radius: 6.0,
            border_size: 1.0
            border_color: (COLOR_SECONDARY)
        }
    }

    SummaryLabel = <Label> {
        width: Fill,
        height: Fit,
        flow: RightWrap
        draw_text: {
            wrap: Word
            text_style: <REGULAR_TEXT>{font_size: 10.5}
            color: (COLOR_TEXT)
        }
        text: ""
    }

    PrefixLabel = <Label> {
        width: 108, height: Fit
        margin: {top: 10}
        draw_text: {
            text_style: <REGULAR_TEXT>{font_size: 10.0}
            color: (COLOR_TEXT)
        }
        text: ""
    }

    PrefixedInputRow = <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 10
        align: {y: 0.5}
    }

    FilterButton = <RobrixIconButton> {
        width: Fit
        padding: 10
        draw_bg: {
            color: (COLOR_BG_PREVIEW)
            border_radius: 4.0
            border_size: 1.0
            border_color: (COLOR_SECONDARY)
        }
        draw_text: {
            color: (COLOR_TEXT)
        }
        icon_walk: { width: 0, height: 0 }
    }

    BindingChip = <RoundedView> {
        width: Fit, height: Fit
        padding: {top: 5, bottom: 5, left: 8, right: 8}
        show_bg: true
        draw_bg: {
            color: (COLOR_BG_PREVIEW)
            border_radius: 10.0
            border_size: 1.0
            border_color: (COLOR_SECONDARY)
        }

        chip_label = <Label> {
            width: Fit, height: Fit
            draw_text: {
                text_style: <REGULAR_TEXT>{font_size: 9.5}
                color: (COLOR_TEXT)
            }
            text: ""
        }
    }

    BindingListEntry = {{BindingListEntry}} {
        width: Fill, height: Fit
        flow: Down
        spacing: 8
        padding: 12
        margin: {bottom: 8}
        show_bg: true
        draw_bg: {
            color: (COLOR_BG_PREVIEW)
            border_radius: 6.0
            border_size: 1.0
            border_color: (COLOR_SECONDARY)
        }

        row_title = <View> {
            width: Fill, height: Fit
            flow: RightWrap
            spacing: 8

            room_name = <Label> {
                width: Fit, height: Fit
                draw_text: {
                    text_style: <THEME_FONT_BOLD>{font_size: 11.0}
                    color: (COLOR_TEXT)
                }
                text: "[Room Name]"
            }

            room_id = <Label> {
                width: Fill, height: Fit
                draw_text: {
                    text_style: <REGULAR_TEXT>{font_size: 9.5}
                    color: (COLOR_TEXT)
                }
                text: "[Room ID]"
            }
        }

        row_meta = <Label> {
            width: Fill, height: Fit
            flow: RightWrap
            draw_text: {
                wrap: Word
                text_style: <REGULAR_TEXT>{font_size: 10.0}
                color: (COLOR_TEXT)
            }
            text: ""
        }

        row_chips = <View> {
            width: Fill, height: Fit
            flow: RightWrap
            spacing: 8

            mode_chip = <BindingChip> { chip_label = { text: "mode" } }
            sender_chip = <BindingChip> { chip_label = { text: "sender" } }
            runtime_chip = <BindingChip> { chip_label = { text: "runtime" } }
        }

        row_actions = <View> {
            width: Fill, height: Fit
            flow: RightWrap
            spacing: 10

            open_room_button = <RobrixIconButton> {
                width: Fit
                padding: 10
                draw_bg: {
                    color: (COLOR_ACTIVE_PRIMARY)
                }
                draw_icon: {
                    svg_file: (ICON_LINK)
                    color: (COLOR_PRIMARY)
                }
                draw_text: {
                    color: (COLOR_PRIMARY)
                }
                icon_walk: { width: 14, height: 14 }
                text: "Open Room"
            }

            manage_sender_button = <RobrixIconButton> {
                width: Fit
                padding: 10
                draw_bg: {
                    color: (COLOR_SECONDARY)
                }
                draw_icon: {
                    svg_file: (ICON_INFO)
                    color: (COLOR_TEXT)
                }
                icon_walk: { width: 14, height: 14 }
                text: "Manage Sender"
            }

            unbind_room_button = <RobrixIconButton> {
                width: Fit
                padding: 10
                draw_bg: {
                    color: (COLOR_BG_DANGER_RED)
                    border_color: (COLOR_FG_DANGER_RED)
                }
                draw_icon: {
                    svg_file: (ICON_CLOSE)
                    color: (COLOR_FG_DANGER_RED)
                }
                draw_text: {
                    color: (COLOR_FG_DANGER_RED)
                }
                icon_walk: { width: 14, height: 14 }
                text: "Unbind"
            }
        }
    }

    SenderDirectoryEntry = {{SenderDirectoryListEntry}} {
        width: Fill, height: Fit
        flow: Down
        spacing: 8
        padding: 12
        margin: {bottom: 8}
        show_bg: true
        draw_bg: {
            color: (COLOR_BG_PREVIEW)
            border_radius: 6.0
            border_size: 1.0
            border_color: (COLOR_SECONDARY)
        }

        sender_title = <View> {
            width: Fill, height: Fit
            flow: RightWrap
            spacing: 8

            sender_name = <Label> {
                width: Fit, height: Fit
                draw_text: {
                    text_style: <THEME_FONT_BOLD>{font_size: 11.0}
                    color: (COLOR_TEXT)
                }
                text: "[Sender Name]"
            }

            sender_id = <Label> {
                width: Fill, height: Fit
                draw_text: {
                    text_style: <REGULAR_TEXT>{font_size: 9.5}
                    color: (COLOR_TEXT)
                }
                text: "[sender-id]"
            }
        }

        sender_chips = <View> {
            width: Fill, height: Fit
            flow: RightWrap
            spacing: 8

            kind_chip = <BindingChip> { chip_label = { text: "matrix-bot" } }
            security_chip = <BindingChip> { chip_label = { text: "standard" } }
            status_chip = <BindingChip> { chip_label = { text: "ready" } }
            default_chip = <BindingChip> { chip_label = { text: "default room sender" } }
        }

        sender_target = <Label> {
            width: Fill, height: Fit
            flow: RightWrap
            draw_text: {
                wrap: Word
                text_style: <REGULAR_TEXT>{font_size: 10.0}
                color: (COLOR_TEXT)
            }
            text: ""
        }

        sender_verification = <Label> {
            width: Fill, height: Fit
            flow: RightWrap
            draw_text: {
                wrap: Word
                text_style: <REGULAR_TEXT>{font_size: 10.0}
                color: (COLOR_TEXT)
            }
            text: ""
        }

        sender_room_access = <Label> {
            width: Fill, height: Fit
            flow: RightWrap
            draw_text: {
                wrap: Word
                text_style: <REGULAR_TEXT>{font_size: 10.0}
                color: (COLOR_TEXT)
            }
            text: ""
        }

        sender_actions = <View> {
            width: Fill, height: Fit
            flow: RightWrap
            spacing: 10

            sender_manage_button = <RobrixIconButton> {
                width: Fit
                padding: 10
                draw_bg: {
                    color: (COLOR_ACTIVE_PRIMARY)
                }
                draw_icon: {
                    svg_file: (ICON_LINK)
                    color: (COLOR_PRIMARY)
                }
                draw_text: {
                    color: (COLOR_PRIMARY)
                }
                icon_walk: { width: 14, height: 14 }
                text: "Manage"
            }

            sender_default_button = <RobrixIconButton> {
                width: Fit
                padding: 10
                draw_bg: {
                    color: (COLOR_SECONDARY)
                }
                draw_icon: {
                    svg_file: (ICON_CHECKMARK)
                    color: (COLOR_TEXT)
                }
                icon_walk: { width: 14, height: 14 }
                text: "Use As Default"
            }
        }
    }

    SectionButton = <RobrixIconButton> {
        width: Fit
        padding: 11
        draw_bg: {
            color: (COLOR_BG_PREVIEW)
        }
        draw_icon: {
            svg_file: (ICON_LINK)
            color: (COLOR_TEXT)
        }
        icon_walk: { width: 14, height: 14 }
    }

    RuntimeToggleButton = <RobrixIconButton> {
        width: Fill
        padding: 12
        spacing: 10
        draw_bg: {
            color: (COLOR_BG_PREVIEW)
            border_radius: 6.0
            border_size: 1.0
            border_color: (COLOR_SECONDARY)
        }
        draw_icon: {
            svg_file: (ICON_COLLAPSE)
            rotation_angle: 90.0
            color: (COLOR_TEXT)
        }
        draw_text: {
            color: (COLOR_TEXT)
        }
        icon_walk: { width: 12, height: 12 }
    }

    RuntimeForm = <View> {
        visible: false
        width: Fill, height: Fit
        flow: Down
        spacing: 8
    }

    RuntimeCardBody = <View> {
        visible: false
        width: Fill, height: Fit
        flow: Down
        spacing: 8
    }

    SenderManagerDropDown = <DropDown> {
        width: Fill
        height: Fit
        popup_menu_position: BelowInput
        labels: ["Create New Sender"]
        draw_bg: {
            border_radius: 6.0
            color: (COLOR_PRIMARY)
            color_hover: (COLOR_BG_PREVIEW)
            color_focus: (COLOR_BG_PREVIEW)
            color_down: (COLOR_BG_PREVIEW)
            border_color: (COLOR_SECONDARY)
            border_color_hover: (COLOR_SECONDARY)
            border_color_focus: (COLOR_ACTIVE_PRIMARY)
            border_color_down: (COLOR_ACTIVE_PRIMARY)
            border_color_2: (COLOR_SECONDARY)
            border_color_2_hover: (COLOR_SECONDARY)
            border_color_2_focus: (COLOR_ACTIVE_PRIMARY)
            border_color_2_down: (COLOR_ACTIVE_PRIMARY)
        }
        draw_text: {
            color: (COLOR_TEXT)
        }
    }

    pub BotfatherSettings = {{BotfatherSettings}} {
        width: Fill, height: Fit
        flow: Down
        spacing: 12
        margin: {bottom: 24}

        <TitleLabel> {
            text: "BotFather Settings"
        }

        <SummaryLabel> {
            text: "BotFather now groups its controls by cards. Use Runtimes to configure transports, Senders to manage Matrix delivery identities, Bindings to inspect room takeover state, Bots to inspect what is installed, and Workspace for shared inventory and root binding."
        }

        section_selector = <View> {
            width: Fill, height: Fit
            flow: RightWrap
            spacing: 10

            runtimes_section_button = <SectionButton> {
                text: "Runtimes"
            }

            bots_section_button = <SectionButton> {
                draw_icon: {
                    svg_file: (ICON_INFO)
                    color: (COLOR_TEXT)
                }
                text: "Bots"
            }

            senders_section_button = <SectionButton> {
                draw_icon: {
                    svg_file: (ICON_LINK)
                    color: (COLOR_TEXT)
                }
                text: "Senders"
            }

            bindings_section_button = <SectionButton> {
                draw_icon: {
                    svg_file: (ICON_CHECKMARK)
                    color: (COLOR_TEXT)
                }
                text: "Bindings"
            }

            workspace_section_button = <SectionButton> {
                draw_icon: {
                    svg_file: (ICON_CHECKMARK)
                    color: (COLOR_TEXT)
                }
                text: "Workspace"
            }

            diagnostics_section_button = <SectionButton> {
                draw_icon: {
                    svg_file: (ICON_INFO)
                    color: (COLOR_TEXT)
                }
                text: "Diagnostics"
            }
        }

        runtimes_section = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 12

            crew_runtime_card = <SettingsCard> {
                crew_runtime_toggle = <RuntimeToggleButton> {
                    text: "Crew Runtime"
                }

                crew_runtime_body = <RuntimeCardBody> {
                    crew_runtime_summary_label = <SummaryLabel> {}

                    <View> {
                        width: Fill, height: Fit
                        flow: RightWrap
                        spacing: 10

                        crew_runtime_healthcheck_button = <RobrixIconButton> {
                            width: Fit
                            padding: 12
                            draw_bg: {
                                color: (COLOR_SECONDARY)
                            }
                            draw_icon: {
                                svg_file: (ICON_INFO)
                                color: (COLOR_TEXT)
                            }
                            icon_walk: { width: 14, height: 14 }
                            text: "Healthcheck"
                        }
                    }

                    crew_runtime_form = <RuntimeForm> {
                        crew_endpoint_input = <SimpleTextInput> {
                            width: Fill, height: Fit
                            empty_text: "http://127.0.0.1:8000"
                        }

                        crew_auth_token_env_input = <SimpleTextInput> {
                            width: Fill, height: Fit
                            empty_text: "Optional bearer token env var, e.g. CREW_API_TOKEN"
                        }
                    }
                }
            }

            openclaw_runtime_card = <SettingsCard> {
                openclaw_runtime_toggle = <RuntimeToggleButton> {
                    text: "OpenClaw Runtime"
                }

                openclaw_runtime_body = <RuntimeCardBody> {
                    openclaw_runtime_summary_label = <SummaryLabel> {}

                    <View> {
                        width: Fill, height: Fit
                        flow: RightWrap
                        spacing: 10

                        openclaw_runtime_healthcheck_button = <RobrixIconButton> {
                            width: Fit
                            padding: 12
                            draw_bg: {
                                color: (COLOR_SECONDARY)
                            }
                            draw_icon: {
                                svg_file: (ICON_INFO)
                                color: (COLOR_TEXT)
                            }
                            icon_walk: { width: 14, height: 14 }
                            text: "Healthcheck"
                        }
                    }

                    openclaw_runtime_form = <RuntimeForm> {
                        openclaw_gateway_input = <SimpleTextInput> {
                            width: Fill, height: Fit
                            empty_text: "ws://127.0.0.1:24282/ws"
                        }

                        openclaw_auth_token_env_input = <SimpleTextInput> {
                            width: Fill, height: Fit
                            empty_text: "Optional gateway token env var"
                        }
                    }
                }
            }
        }

        bots_section = <View> {
            visible: false
            width: Fill, height: Fit
            flow: Down
            spacing: 12

            bots_inventory_card = <SettingsCard> {
                <SubsectionLabel> {
                    text: "Installed Bots"
                }

                bots_summary_label = <SummaryLabel> {}
            }

            room_stream_mode_card = <SettingsCard> {
                <SubsectionLabel> {
                    text: "Room Stream Mode"
                }

                room_stream_mode_summary_label = <SummaryLabel> {}

                room_stream_mode_toggle = <RobrixIconButton> {
                    width: Fit
                    padding: 12
                    draw_bg: {
                        color: (COLOR_BG_PREVIEW)
                    }
                    draw_icon: {
                        svg_file: (ICON_LINK)
                        color: (COLOR_TEXT)
                    }
                    icon_walk: { width: 14, height: 14 }
                    text: "Enable Preview"
                }
            }

            bot_commands_card = <SettingsCard> {
                <SubsectionLabel> {
                    text: "Slash Commands"
                }

                command_hint_label = <SummaryLabel> {}
            }
        }

        senders_section = <View> {
            visible: false
            width: Fill, height: Fit
            flow: Down
            spacing: 12

            sender_directory_card = <SettingsCard> {
                <SubsectionLabel> {
                    text: "Configured Senders"
                }

                <SummaryLabel> {
                    text: "Configured Matrix bot senders appear here as cards. Current user delivery stays available in per-room binding, but it is no longer shown as a separate sender card."
                }

                sender_directory_list_shell = <RoundedView> {
                    width: Fill, height: Fit
                    padding: 10
                    show_bg: true
                    draw_bg: {
                        color: (COLOR_PRIMARY)
                        border_radius: 6.0
                        border_size: 1.0
                        border_color: (COLOR_SECONDARY)
                    }

                    sender_directory_list = <FlatList> {
                        width: Fill
                        height: 280
                        spacing: 0
                        flow: Down
                        grab_key_focus: true
                        drag_scrolling: true
                        scroll_bars: { show_scroll_x: false, show_scroll_y: true }

                        sender_entry = <SenderDirectoryEntry> {}
                    }

                    sender_directory_empty_label = <SummaryLabel> {
                        text: "No Matrix bot senders are configured yet."
                    }
                }
            }

            shared_sender_card = <SettingsCard> {
                shared_sender_toggle = <RuntimeToggleButton> {
                    text: "Shared Bot Sender"
                }

                shared_sender_body = <RuntimeCardBody> {
                    shared_sender_summary_label = <SummaryLabel> {}

                    shared_sender_form = <RuntimeForm> {
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Homeserver" }
                            shared_sender_homeserver_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "https://matrix.example.org"
                            }
                        }
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Matrix ID" }
                            shared_sender_user_id_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "@botfather:example.org"
                            }
                        }
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Device ID" }
                            shared_sender_device_id_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "BOTDEVICE01"
                            }
                        }
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Token Env" }
                            shared_sender_access_token_env_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "BOTFATHER_MATRIX_ACCESS_TOKEN"
                            }
                        }
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Password" }
                            shared_sender_password_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "Matrix account password (used only for verification)"
                                is_password: true
                            }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit
                        flow: RightWrap
                        spacing: 10

                        shared_sender_verify_button = <RobrixIconButton> {
                            width: Fit
                            padding: 12
                            draw_bg: {
                                color: (COLOR_ACTIVE_PRIMARY)
                            }
                            draw_icon: {
                                svg_file: (ICON_CHECKMARK)
                                color: (COLOR_PRIMARY)
                            }
                            draw_text: {
                                color: (COLOR_PRIMARY)
                            }
                            icon_walk: { width: 14, height: 14 }
                            text: "Verify Login"
                        }
                    }
                }
            }

            secure_sender_card = <SettingsCard> {
                secure_sender_toggle = <RuntimeToggleButton> {
                    text: "Secure Room Sender"
                }

                secure_sender_body = <RuntimeCardBody> {
                    secure_sender_summary_label = <SummaryLabel> {}

                    secure_sender_form = <RuntimeForm> {
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Homeserver" }
                            secure_sender_homeserver_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "https://matrix.example.org"
                            }
                        }
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Matrix ID" }
                            secure_sender_user_id_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "@secure-bot:example.org"
                            }
                        }
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Device ID" }
                            secure_sender_device_id_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "SECUREBOT01"
                            }
                        }
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Token Env" }
                            secure_sender_access_token_env_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "SECURE_ROOM_BOT_ACCESS_TOKEN"
                            }
                        }
                        <PrefixedInputRow> {
                            <PrefixLabel> { text: "Password" }
                            secure_sender_password_input = <SimpleTextInput> {
                                width: Fill, height: Fit
                                empty_text: "Matrix account password (used only for verification)"
                                is_password: true
                            }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit
                        flow: RightWrap
                        spacing: 10

                        secure_sender_verify_button = <RobrixIconButton> {
                            width: Fit
                            padding: 12
                            draw_bg: {
                                color: (COLOR_ACTIVE_PRIMARY)
                            }
                            draw_icon: {
                                svg_file: (ICON_CHECKMARK)
                                color: (COLOR_PRIMARY)
                            }
                            draw_text: {
                                color: (COLOR_PRIMARY)
                            }
                            icon_walk: { width: 14, height: 14 }
                            text: "Verify Login"
                        }
                    }
                }
            }

            custom_sender_card = <SettingsCard> {
                <SubsectionLabel> {
                    text: "Custom Sender Manager"
                }

                selected_custom_sender_summary_label = <SummaryLabel> {
                    text: "Create a new sender profile or select an existing one."
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Sender" }
                    custom_sender_selector_dropdown = <SenderManagerDropDown> {}
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Profile ID" }
                    custom_sender_profile_id_input = <SimpleTextInput> {
                        width: Fill, height: Fit
                        empty_text: "qa-shared-bot"
                    }
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Display Name" }
                    custom_sender_name_input = <SimpleTextInput> {
                        width: Fill, height: Fit
                        empty_text: "QA Shared Bot"
                    }
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Security" }
                    custom_sender_security_dropdown = <SenderManagerDropDown> {
                        labels: ["Standard", "Elevated", "Isolated"]
                    }
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Homeserver" }
                    custom_sender_homeserver_input = <SimpleTextInput> {
                        width: Fill, height: Fit
                        empty_text: "https://matrix.example.org"
                    }
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Matrix ID" }
                    custom_sender_user_id_input = <SimpleTextInput> {
                        width: Fill, height: Fit
                        empty_text: "@qa-bot:example.org"
                    }
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Device ID" }
                    custom_sender_device_id_input = <SimpleTextInput> {
                        width: Fill, height: Fit
                        empty_text: "QABOT01"
                    }
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Token Env" }
                    custom_sender_access_token_env_input = <SimpleTextInput> {
                        width: Fill, height: Fit
                        empty_text: "QA_BOT_MATRIX_ACCESS_TOKEN"
                    }
                }

                <PrefixedInputRow> {
                    <PrefixLabel> { text: "Password" }
                    custom_sender_password_input = <SimpleTextInput> {
                        width: Fill, height: Fit
                        empty_text: "Matrix account password (used only for verification)"
                        is_password: true
                    }
                }

                <SummaryLabel> {
                    text: "Independent Matrix senders do not auto-join rooms. Invite the bot account first, or manually join it in public rooms before using it as a room sender."
                }

                <View> {
                    width: Fill, height: Fit
                    flow: RightWrap
                    spacing: 10

                    new_custom_sender_button = <RobrixIconButton> {
                        width: Fit
                        padding: 12
                        draw_bg: {
                            color: (COLOR_SECONDARY)
                        }
                        draw_icon: {
                            svg_file: (ICON_INFO)
                            color: (COLOR_TEXT)
                        }
                        icon_walk: { width: 14, height: 14 }
                        text: "New Sender"
                    }

                    save_custom_sender_button = <RobrixIconButton> {
                        width: Fit
                        padding: 12
                        draw_bg: {
                            color: (COLOR_ACTIVE_PRIMARY)
                        }
                        draw_icon: {
                            svg_file: (ICON_CHECKMARK)
                            color: (COLOR_PRIMARY)
                        }
                        draw_text: {
                            color: (COLOR_PRIMARY)
                        }
                        icon_walk: { width: 14, height: 14 }
                        text: "Save Sender"
                    }

                    verify_custom_sender_button = <RobrixIconButton> {
                        width: Fit
                        padding: 12
                        draw_bg: {
                            color: (COLOR_ACTIVE_PRIMARY)
                        }
                        draw_icon: {
                            svg_file: (ICON_CHECKMARK)
                            color: (COLOR_PRIMARY)
                        }
                        draw_text: {
                            color: (COLOR_PRIMARY)
                        }
                        icon_walk: { width: 14, height: 14 }
                        text: "Verify Login"
                    }

                    set_default_custom_sender_button = <RobrixIconButton> {
                        width: Fit
                        padding: 12
                        draw_bg: {
                            color: (COLOR_SECONDARY)
                        }
                        draw_icon: {
                            svg_file: (ICON_LINK)
                            color: (COLOR_TEXT)
                        }
                        icon_walk: { width: 14, height: 14 }
                        text: "Use As Default"
                    }

                    delete_custom_sender_button = <RobrixIconButton> {
                        width: Fit
                        padding: 12
                        draw_bg: {
                            color: (COLOR_BG_DANGER_RED)
                            border_color: (COLOR_FG_DANGER_RED)
                        }
                        draw_icon: {
                            svg_file: (ICON_CLOSE)
                            color: (COLOR_FG_DANGER_RED)
                        }
                        draw_text: {
                            color: (COLOR_FG_DANGER_RED)
                        }
                        icon_walk: { width: 14, height: 14 }
                        text: "Delete Sender"
                    }
                }
            }
        }

        bindings_section = <View> {
            visible: false
            width: Fill, height: Fit
            flow: Down
            spacing: 12

            bindings_card = <SettingsCard> {
                <SubsectionLabel> {
                    text: "Managed Room Bindings"
                }

                bindings_stats_label = <SummaryLabel> {}

                <SummaryLabel> {
                    text: "Only rooms that you explicitly bind from the room panel appear here. Rooms merely resolved by default bot settings stay out of this list."
                }

                <View> {
                    width: Fill, height: Fit
                    flow: RightWrap
                    spacing: 10

                    bindings_filter_all_button = <FilterButton> {
                        text: "All"
                    }
                    bindings_filter_shared_button = <FilterButton> {
                        text: "Shared"
                    }
                    bindings_filter_personal_button = <FilterButton> {
                        text: "Personal"
                    }
                }

                bindings_list_shell = <RoundedView> {
                    width: Fill, height: Fit
                    padding: 10
                    show_bg: true
                    draw_bg: {
                        color: (COLOR_PRIMARY)
                        border_radius: 6.0
                        border_size: 1.0
                        border_color: (COLOR_SECONDARY)
                    }

                    bindings_list = <FlatList> {
                        width: Fill
                        height: 260
                        spacing: 0
                        flow: Down
                        grab_key_focus: true
                        drag_scrolling: true
                        scroll_bars: { show_scroll_x: false, show_scroll_y: true }

                        binding_entry = <BindingListEntry> {}
                    }

                    bindings_empty_label = <SummaryLabel> {
                        text: "No bindings match the current filter."
                    }
                }
            }
        }

        workspace_section = <View> {
            visible: false
            width: Fill, height: Fit
            flow: Down
            spacing: 12

            workspace_card = <SettingsCard> {
                <SubsectionLabel> {
                    text: "Shared Workspace"
                }

                workspace_summary_label = <SummaryLabel> {}

                workspace_root_input = <SimpleTextInput> {
                    width: Fill, height: Fit
                    empty_text: "/path/to/workspace (optional)"
                }
            }
        }

        diagnostics_section = <View> {
            visible: false
            width: Fill, height: Fit
            flow: Down
            spacing: 12

            diagnostics_card = <SettingsCard> {
                <SubsectionLabel> {
                    text: "State Diagnostics"
                }

                diagnostics_summary_label = <SummaryLabel> {}
            }
        }

        actions_card = <SettingsCard> {
            <SubsectionLabel> {
                text: "Actions"
            }

            <View> {
                width: Fill, height: Fit
                flow: RightWrap
                spacing: 10

                refresh_inventory_button = <RobrixIconButton> {
                    width: Fit
                    padding: 12
                    draw_bg: {
                        color: (COLOR_SECONDARY)
                    }
                    draw_icon: {
                        svg_file: (ICON_ROTATE_CW)
                        color: (COLOR_TEXT)
                    }
                    icon_walk: { width: 14, height: 14 }
                    text: "Refresh Inventory"
                }

                save_defaults_button = <RobrixIconButton> {
                    width: Fit
                    padding: 12
                    draw_bg: {
                        color: (COLOR_ACTIVE_PRIMARY)
                    }
                    draw_icon: {
                        svg_file: (ICON_CHECKMARK)
                        color: (COLOR_PRIMARY)
                    }
                    draw_text: {
                        color: (COLOR_PRIMARY)
                    }
                    icon_walk: { width: 14, height: 14 }
                    text: "Save BotFather Config"
                }
            }

            status_label = <SummaryLabel> {}
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BotfatherSettingsSection {
    #[default]
    Runtimes,
    Bots,
    Senders,
    Bindings,
    Workspace,
    Diagnostics,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BindingsFilter {
    #[default]
    All,
    Shared,
    Personal,
}

#[derive(Clone, Debug, DefaultNone)]
enum BindingEntryAction {
    OpenRoom(String),
    ManageSender(String),
    Unbind(String),
    None,
}

#[derive(Clone, Debug, DefaultNone)]
enum SenderEntryAction {
    Manage(String),
    MakeDefault(String),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct BindingListEntry {
    #[deref]
    view: View,
    #[rust]
    entry: Option<botfather::ExplicitRoomBindingEntry>,
}

impl Widget for BindingListEntry {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        let Some(entry) = self.entry.as_ref() else {
            return;
        };
        if let Event::Actions(actions) = event {
            if self.view.button(ids!(open_room_button)).clicked(actions) {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    BindingEntryAction::OpenRoom(entry.room_id.clone()),
                );
            }
            if self
                .view
                .button(ids!(manage_sender_button))
                .clicked(actions)
            {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    BindingEntryAction::ManageSender(entry.sender_profile_id.clone()),
                );
            }
            if self.view.button(ids!(unbind_room_button)).clicked(actions) {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    BindingEntryAction::Unbind(entry.room_id.clone()),
                );
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // FlatList can instantiate the template once before item props are attached.
        let Some(entry) = scope.props.get::<botfather::ExplicitRoomBindingEntry>() else {
            self.entry = None;
            return DrawStep::done();
        };
        if self.entry.as_ref().is_none_or(|existing| existing != entry) {
            self.entry = Some(entry.clone());
        }

        self.view
            .label(ids!(room_name))
            .set_text(cx, &entry.room_name);
        self.view.label(ids!(room_id)).set_text(cx, &entry.room_id);
        self.view.label(ids!(row_meta)).set_text(
            cx,
            &format!(
                "bot: {} | sender: {} | source: {}",
                entry.bot_name, entry.sender_name, entry.source
            ),
        );
        self.view
            .label(ids!(mode_chip.chip_label))
            .set_text(cx, &entry.mode);
        self.view.label(ids!(sender_chip.chip_label)).set_text(
            cx,
            &format!(
                "{} / {}",
                entry.sender_security,
                if entry.sender_ready {
                    "ready"
                } else {
                    "needs setup"
                }
            ),
        );
        self.view
            .label(ids!(runtime_chip.chip_label))
            .set_text(cx, &entry.runtime);
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SenderDirectoryListEntry {
    #[deref]
    view: View,
    #[rust]
    entry: Option<botfather::SenderDirectoryEntry>,
}

impl Widget for SenderDirectoryListEntry {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        let Some(entry) = self.entry.as_ref() else {
            return;
        };
        if let Event::Actions(actions) = event {
            if self.view.button(ids!(sender_manage_button)).clicked(actions) {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    SenderEntryAction::Manage(entry.sender_profile_id.clone()),
                );
            }
            if self
                .view
                .button(ids!(sender_default_button))
                .clicked(actions)
            {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    SenderEntryAction::MakeDefault(entry.sender_profile_id.clone()),
                );
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let Some(entry) = scope.props.get::<botfather::SenderDirectoryEntry>() else {
            self.entry = None;
            return DrawStep::done();
        };
        if self.entry.as_ref().is_none_or(|existing| existing != entry) {
            self.entry = Some(entry.clone());
        }

        self.view
            .label(ids!(sender_name))
            .set_text(cx, &entry.name);
        self.view
            .label(ids!(sender_id))
            .set_text(cx, &entry.sender_profile_id);
        self.view
            .label(ids!(kind_chip.chip_label))
            .set_text(cx, &entry.kind);
        self.view
            .label(ids!(security_chip.chip_label))
            .set_text(cx, &entry.security);
        self.view.label(ids!(status_chip.chip_label)).set_text(
            cx,
            if entry.ready {
                "ready"
            } else {
                "needs setup"
            },
        );
        self.view
            .view(ids!(default_chip))
            .set_visible(cx, entry.is_default);
        self.view
            .label(ids!(sender_target))
            .set_text(cx, &format!("target: {}", entry.target));
        self.view
            .label(ids!(sender_verification))
            .set_text(cx, &format!("verification: {}", entry.verification));
        self.view
            .label(ids!(sender_room_access))
            .set_text(cx, &format!("room access: {}", entry.room_access));
        self.view
            .button(ids!(sender_default_button))
            .set_visible(cx, entry.can_set_default && !entry.is_default);
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct BotfatherSettings {
    #[deref]
    view: View,
    #[rust]
    selected_section: BotfatherSettingsSection,
    #[rust]
    crew_runtime_expanded: bool,
    #[rust]
    openclaw_runtime_expanded: bool,
    #[rust]
    shared_sender_expanded: bool,
    #[rust]
    secure_sender_expanded: bool,
    #[rust]
    custom_sender_choice_ids: Vec<String>,
    #[rust]
    selected_custom_sender_profile_id: Option<String>,
    #[rust]
    selected_custom_sender_security: SenderSecurityLevel,
    #[rust]
    sender_directory_entries: Vec<botfather::SenderDirectoryEntry>,
    #[rust]
    bindings_filter: BindingsFilter,
    #[rust]
    binding_entries: Vec<botfather::ExplicitRoomBindingEntry>,
}

impl Widget for BotfatherSettings {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if matches!(event, Event::Signal) {
            let _ = botfather::refresh_inventory_from_rooms_list(cx);
        }

        if let Event::Actions(actions) = event {
            let runtimes_section_button = self.view.button(ids!(runtimes_section_button));
            let bots_section_button = self.view.button(ids!(bots_section_button));
            let senders_section_button = self.view.button(ids!(senders_section_button));
            let bindings_section_button = self.view.button(ids!(bindings_section_button));
            let workspace_section_button = self.view.button(ids!(workspace_section_button));
            let diagnostics_section_button = self.view.button(ids!(diagnostics_section_button));
            let refresh_inventory_button = self.view.button(ids!(refresh_inventory_button));
            let save_defaults_button = self.view.button(ids!(save_defaults_button));
            let bindings_filter_all_button = self.view.button(ids!(bindings_filter_all_button));
            let bindings_filter_shared_button =
                self.view.button(ids!(bindings_filter_shared_button));
            let bindings_filter_personal_button =
                self.view.button(ids!(bindings_filter_personal_button));
            let room_stream_mode_toggle = self.view.button(ids!(room_stream_mode_toggle));
            let crew_runtime_toggle = self.view.button(ids!(crew_runtime_toggle));
            let openclaw_runtime_toggle = self.view.button(ids!(openclaw_runtime_toggle));
            let shared_sender_toggle = self.view.button(ids!(shared_sender_toggle));
            let secure_sender_toggle = self.view.button(ids!(secure_sender_toggle));
            let shared_sender_verify_button = self.view.button(ids!(shared_sender_verify_button));
            let secure_sender_verify_button = self.view.button(ids!(secure_sender_verify_button));
            let new_custom_sender_button = self.view.button(ids!(new_custom_sender_button));
            let save_custom_sender_button = self.view.button(ids!(save_custom_sender_button));
            let verify_custom_sender_button =
                self.view.button(ids!(verify_custom_sender_button));
            let set_default_custom_sender_button =
                self.view.button(ids!(set_default_custom_sender_button));
            let delete_custom_sender_button =
                self.view.button(ids!(delete_custom_sender_button));
            let crew_runtime_healthcheck_button =
                self.view.button(ids!(crew_runtime_healthcheck_button));
            let openclaw_runtime_healthcheck_button =
                self.view.button(ids!(openclaw_runtime_healthcheck_button));
            let crew_endpoint_input = self.view.text_input(ids!(crew_endpoint_input));
            let crew_auth_token_env_input = self.view.text_input(ids!(crew_auth_token_env_input));
            let openclaw_gateway_input = self.view.text_input(ids!(openclaw_gateway_input));
            let openclaw_auth_token_env_input =
                self.view.text_input(ids!(openclaw_auth_token_env_input));
            let workspace_root_input = self.view.text_input(ids!(workspace_root_input));
            let shared_sender_homeserver_input =
                self.view.text_input(ids!(shared_sender_homeserver_input));
            let shared_sender_user_id_input =
                self.view.text_input(ids!(shared_sender_user_id_input));
            let shared_sender_device_id_input =
                self.view.text_input(ids!(shared_sender_device_id_input));
            let shared_sender_access_token_env_input = self
                .view
                .text_input(ids!(shared_sender_access_token_env_input));
            let shared_sender_password_input =
                self.view.text_input(ids!(shared_sender_password_input));
            let secure_sender_homeserver_input =
                self.view.text_input(ids!(secure_sender_homeserver_input));
            let secure_sender_user_id_input =
                self.view.text_input(ids!(secure_sender_user_id_input));
            let secure_sender_device_id_input =
                self.view.text_input(ids!(secure_sender_device_id_input));
            let secure_sender_access_token_env_input = self
                .view
                .text_input(ids!(secure_sender_access_token_env_input));
            let secure_sender_password_input =
                self.view.text_input(ids!(secure_sender_password_input));
            let custom_sender_selector_dropdown =
                self.view.drop_down(ids!(custom_sender_selector_dropdown));
            let custom_sender_security_dropdown =
                self.view.drop_down(ids!(custom_sender_security_dropdown));
            let custom_sender_profile_id_input =
                self.view.text_input(ids!(custom_sender_profile_id_input));
            let custom_sender_name_input =
                self.view.text_input(ids!(custom_sender_name_input));
            let custom_sender_homeserver_input =
                self.view.text_input(ids!(custom_sender_homeserver_input));
            let custom_sender_user_id_input =
                self.view.text_input(ids!(custom_sender_user_id_input));
            let custom_sender_device_id_input =
                self.view.text_input(ids!(custom_sender_device_id_input));
            let custom_sender_access_token_env_input = self
                .view
                .text_input(ids!(custom_sender_access_token_env_input));
            let custom_sender_password_input =
                self.view.text_input(ids!(custom_sender_password_input));

            if runtimes_section_button.clicked(actions) {
                self.set_section(cx, BotfatherSettingsSection::Runtimes);
            }
            if bots_section_button.clicked(actions) {
                self.set_section(cx, BotfatherSettingsSection::Bots);
            }
            if senders_section_button.clicked(actions) {
                self.set_section(cx, BotfatherSettingsSection::Senders);
            }
            if bindings_section_button.clicked(actions) {
                self.set_section(cx, BotfatherSettingsSection::Bindings);
            }
            if workspace_section_button.clicked(actions) {
                self.set_section(cx, BotfatherSettingsSection::Workspace);
            }
            if diagnostics_section_button.clicked(actions) {
                self.set_section(cx, BotfatherSettingsSection::Diagnostics);
            }
            if bindings_filter_all_button.clicked(actions) {
                self.bindings_filter = BindingsFilter::All;
                self.apply_bindings_filter_style(cx);
            }
            if bindings_filter_shared_button.clicked(actions) {
                self.bindings_filter = BindingsFilter::Shared;
                self.apply_bindings_filter_style(cx);
            }
            if bindings_filter_personal_button.clicked(actions) {
                self.bindings_filter = BindingsFilter::Personal;
                self.apply_bindings_filter_style(cx);
            }
            if crew_runtime_toggle.clicked(actions) {
                self.crew_runtime_expanded = !self.crew_runtime_expanded;
                self.apply_runtime_cards_state(cx);
            }
            if openclaw_runtime_toggle.clicked(actions) {
                self.openclaw_runtime_expanded = !self.openclaw_runtime_expanded;
                self.apply_runtime_cards_state(cx);
            }
            if shared_sender_toggle.clicked(actions) {
                self.shared_sender_expanded = !self.shared_sender_expanded;
                self.apply_runtime_cards_state(cx);
            }
            if secure_sender_toggle.clicked(actions) {
                self.secure_sender_expanded = !self.secure_sender_expanded;
                self.apply_runtime_cards_state(cx);
            }
            if crew_runtime_healthcheck_button.clicked(actions) {
                match botfather::run_runtime_healthcheck(RuntimeKind::Crew) {
                    Ok(()) => self.set_status(cx, "Running Crew runtime healthcheck..."),
                    Err(error) => self.set_status(cx, &error),
                }
            }
            if openclaw_runtime_healthcheck_button.clicked(actions) {
                match botfather::run_runtime_healthcheck(RuntimeKind::OpenClaw) {
                    Ok(()) => self.set_status(cx, "Running OpenClaw runtime healthcheck..."),
                    Err(error) => self.set_status(cx, &error),
                }
            }
            if shared_sender_verify_button.clicked(actions) {
                let homeserver = shared_sender_homeserver_input.text();
                let user_id = shared_sender_user_id_input.text();
                let password = shared_sender_password_input.text();
                let access_token_env = shared_sender_access_token_env_input.text();
                self.set_status(cx, "Verifying Shared Bot Sender login...");
                spawn_on_tokio(async move {
                    let status = match botfather::sender::verify_sender_login(
                        "shared-bot-sender",
                        &homeserver,
                        &user_id,
                        &password,
                    )
                    .await
                    {
                        Ok(session) => match botfather::save_verified_sender_session(
                            "shared-bot-sender",
                            &homeserver,
                            &session.matrix_user_id,
                            &session.device_id,
                            &session.access_token,
                            &access_token_env,
                        ) {
                            Ok(message) => message,
                            Err(error) => error,
                        },
                        Err(error) => {
                            let _ = botfather::record_sender_verification_failure(
                                "shared-bot-sender",
                                &homeserver,
                                &user_id,
                                &access_token_env,
                                &error,
                            );
                            error
                        }
                    };
                    Cx::post_action(BotfatherAction::Status(status));
                });
            }
            if secure_sender_verify_button.clicked(actions) {
                let homeserver = secure_sender_homeserver_input.text();
                let user_id = secure_sender_user_id_input.text();
                let password = secure_sender_password_input.text();
                let access_token_env = secure_sender_access_token_env_input.text();
                self.set_status(cx, "Verifying Secure Room Sender login...");
                spawn_on_tokio(async move {
                    let status = match botfather::sender::verify_sender_login(
                        "secure-room-bot-sender",
                        &homeserver,
                        &user_id,
                        &password,
                    )
                    .await
                    {
                        Ok(session) => match botfather::save_verified_sender_session(
                            "secure-room-bot-sender",
                            &homeserver,
                            &session.matrix_user_id,
                            &session.device_id,
                            &session.access_token,
                            &access_token_env,
                        ) {
                            Ok(message) => message,
                            Err(error) => error,
                        },
                        Err(error) => {
                            let _ = botfather::record_sender_verification_failure(
                                "secure-room-bot-sender",
                                &homeserver,
                                &user_id,
                                &access_token_env,
                                &error,
                            );
                            error
                        }
                    };
                    Cx::post_action(BotfatherAction::Status(status));
                });
            }
            if let Some(selected_index) = custom_sender_selector_dropdown.selected(actions) {
                let sender_profile_id = self
                    .custom_sender_choice_ids
                    .get(selected_index)
                    .cloned()
                    .unwrap_or_default();
                if sender_profile_id.is_empty() {
                    self.load_custom_sender_form(cx, None);
                } else {
                    self.load_custom_sender_form(cx, Some(&sender_profile_id));
                    self.set_status(
                        cx,
                        &format!("Loaded sender profile `{sender_profile_id}`."),
                    );
                }
            }
            if let Some(selected_index) = custom_sender_security_dropdown.selected(actions) {
                self.selected_custom_sender_security =
                    sender_security_level_from_index(selected_index);
            }
            if new_custom_sender_button.clicked(actions) {
                self.load_custom_sender_form(cx, None);
                self.set_status(cx, "Preparing a new sender profile.");
            }
            if save_custom_sender_button.clicked(actions) {
                let sender_profile_id = custom_sender_profile_id_input.text();
                let normalized_sender_profile_id =
                    normalize_sender_profile_id_input(&sender_profile_id);
                match botfather::save_sender_profile(
                    &sender_profile_id,
                    &custom_sender_name_input.text(),
                    self.selected_custom_sender_security,
                    &custom_sender_homeserver_input.text(),
                    &custom_sender_user_id_input.text(),
                    &custom_sender_device_id_input.text(),
                    &custom_sender_access_token_env_input.text(),
                ) {
                    Ok(message) => {
                        self.selected_custom_sender_profile_id =
                            (!normalized_sender_profile_id.is_empty())
                                .then_some(normalized_sender_profile_id);
                        self.populate(cx, None);
                        self.set_status(cx, &message);
                    }
                    Err(error) => self.set_status(cx, &error),
                }
            }
            if verify_custom_sender_button.clicked(actions) {
                let sender_profile_id =
                    normalize_sender_profile_id_input(&custom_sender_profile_id_input.text());
                let sender_name = custom_sender_name_input.text();
                let homeserver = custom_sender_homeserver_input.text();
                let user_id = custom_sender_user_id_input.text();
                let device_id = custom_sender_device_id_input.text();
                let password = custom_sender_password_input.text();
                let access_token_env = custom_sender_access_token_env_input.text();
                let security = self.selected_custom_sender_security;
                self.selected_custom_sender_profile_id =
                    (!sender_profile_id.is_empty()).then_some(sender_profile_id.clone());
                self.set_status(cx, "Saving and verifying custom sender login...");
                spawn_on_tokio(async move {
                    let status = match botfather::save_sender_profile(
                        &sender_profile_id,
                        &sender_name,
                        security,
                        &homeserver,
                        &user_id,
                        &device_id,
                        &access_token_env,
                    ) {
                        Ok(_) => match botfather::sender::verify_sender_login(
                            &sender_profile_id,
                            &homeserver,
                            &user_id,
                            &password,
                        )
                        .await
                        {
                            Ok(session) => match botfather::save_verified_sender_session(
                                &sender_profile_id,
                                &homeserver,
                                &session.matrix_user_id,
                                &session.device_id,
                                &session.access_token,
                                &access_token_env,
                            ) {
                                Ok(message) => message,
                                Err(error) => error,
                            },
                            Err(error) => {
                                let _ = botfather::record_sender_verification_failure(
                                    &sender_profile_id,
                                    &homeserver,
                                    &user_id,
                                    &access_token_env,
                                    &error,
                                );
                                error
                            }
                        },
                        Err(error) => error,
                    };
                    Cx::post_action(BotfatherAction::Status(status));
                });
            }
            if set_default_custom_sender_button.clicked(actions) {
                let sender_profile_id =
                    normalize_sender_profile_id_input(&custom_sender_profile_id_input.text());
                match botfather::set_default_room_sender_profile(&sender_profile_id) {
                    Ok(message) => {
                        self.selected_custom_sender_profile_id =
                            (!sender_profile_id.is_empty()).then_some(sender_profile_id);
                        self.populate(cx, None);
                        self.set_status(cx, &message);
                    }
                    Err(error) => self.set_status(cx, &error),
                }
            }
            if delete_custom_sender_button.clicked(actions) {
                let sender_profile_id =
                    normalize_sender_profile_id_input(&custom_sender_profile_id_input.text());
                match botfather::delete_sender_profile(&sender_profile_id) {
                    Ok(message) => {
                        self.selected_custom_sender_profile_id = None;
                        self.populate(cx, None);
                        self.set_status(cx, &message);
                    }
                    Err(error) => self.set_status(cx, &error),
                }
            }
            if room_stream_mode_toggle.clicked(actions) {
                let enable_preview = !botfather::room_stream_preview_enabled();
                match botfather::set_room_stream_preview_enabled(enable_preview) {
                    Ok(message) => {
                        self.populate(cx, None);
                        self.set_status(cx, &message);
                    }
                    Err(error) => self.set_status(cx, &error),
                }
            }

            if refresh_inventory_button.clicked(actions) {
                match botfather::refresh_inventory_from_rooms_list(cx) {
                    Ok(true) => {
                        self.populate(cx, None);
                        self.set_status(cx, "Bot inventory refreshed from the current room list.");
                    }
                    Ok(false) => self.set_status(cx, "Bot inventory was already up to date."),
                    Err(error) => self.set_status(cx, &error),
                }
            }

            if save_defaults_button.clicked(actions) {
                match botfather::save_default_profiles(
                    &crew_endpoint_input.text(),
                    &crew_auth_token_env_input.text(),
                    &openclaw_gateway_input.text(),
                    &openclaw_auth_token_env_input.text(),
                    &workspace_root_input.text(),
                    &shared_sender_homeserver_input.text(),
                    &shared_sender_user_id_input.text(),
                    &shared_sender_device_id_input.text(),
                    &shared_sender_access_token_env_input.text(),
                    &secure_sender_homeserver_input.text(),
                    &secure_sender_user_id_input.text(),
                    &secure_sender_device_id_input.text(),
                    &secure_sender_access_token_env_input.text(),
                ) {
                    Ok(()) => {
                        self.populate(cx, None);
                        self.set_status(
                            cx,
                            "Saved the BotFather runtime and sender configuration.",
                        );
                    }
                    Err(error) => self.set_status(cx, &error),
                }
            }

            for action in actions {
                if let Some(LoginAction::LoginSuccess) = action.downcast_ref() {
                    let _ = botfather::ensure_loaded_for_current_user();
                    self.populate(cx, None);
                    continue;
                }

                if let Some(LogoutAction::ClearAppState { .. }) = action.downcast_ref() {
                    self.clear(cx);
                    continue;
                }

                if let Some(BotfatherAction::StateChanged) = action.downcast_ref() {
                    self.populate(cx, None);
                    continue;
                }

                if let Some(BotfatherAction::Status(status)) = action.downcast_ref() {
                    self.set_status(cx, status);
                }

                if let BindingEntryAction::OpenRoom(room_id) = action.as_widget_action().cast() {
                    if let Ok(room_id) = room_id.parse::<OwnedRoomId>() {
                        cx.action(AppStateAction::NavigateToRoom {
                            room_to_close: None,
                            destination_room: BasicRoomDetails::RoomId(RoomNameId::empty(room_id)),
                        });
                    } else {
                        self.set_status(cx, "Failed to parse room id for navigation.");
                    }
                }
                if let BindingEntryAction::ManageSender(sender_profile_id) =
                    action.as_widget_action().cast()
                {
                    self.focus_sender_profile(cx, &sender_profile_id);
                }
                if let SenderEntryAction::Manage(sender_profile_id) =
                    action.as_widget_action().cast()
                {
                    self.focus_sender_profile(cx, &sender_profile_id);
                }
                if let SenderEntryAction::MakeDefault(sender_profile_id) =
                    action.as_widget_action().cast()
                {
                    match botfather::set_default_room_sender_profile(&sender_profile_id) {
                        Ok(message) => {
                            self.selected_custom_sender_profile_id = Some(sender_profile_id);
                            self.populate(cx, None);
                            self.set_status(cx, &message);
                        }
                        Err(error) => self.set_status(cx, &error),
                    }
                }
                if let BindingEntryAction::Unbind(room_id) = action.as_widget_action().cast() {
                    match botfather::unbind_room(&room_id) {
                        Ok(()) => {
                            self.populate(cx, None);
                            self.set_status(cx, "Removed the room-level bot override.");
                        }
                        Err(error) => self.set_status(cx, &error),
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let bindings_list_uid = self.view.widget(ids!(bindings_list)).widget_uid();
        let sender_directory_list_uid = self.view.widget(ids!(sender_directory_list)).widget_uid();

        while let Some(subview) = self.view.draw_walk(cx, scope, walk).step() {
            let subview_uid = subview.widget_uid();
            if subview_uid == bindings_list_uid {
                let filtered_entries = self.filtered_binding_entries();
                self.view
                    .label(ids!(bindings_empty_label))
                    .set_visible(cx, filtered_entries.is_empty());

                let flat_list_ref = subview.as_flat_list();
                let Some(mut list) = flat_list_ref.borrow_mut() else {
                    continue;
                };
                for entry in filtered_entries {
                    let item_id = LiveId::from_str(&entry.room_id);
                    let Some(item) = list.item(cx, item_id, id!(binding_entry)) else {
                        continue;
                    };
                    let mut item_scope = Scope::with_props(&entry);
                    item.draw_all(cx, &mut item_scope);
                }
                continue;
            }

            if subview_uid == sender_directory_list_uid {
                self.view
                    .label(ids!(sender_directory_empty_label))
                    .set_visible(cx, self.sender_directory_entries.is_empty());

                let flat_list_ref = subview.as_flat_list();
                let Some(mut list) = flat_list_ref.borrow_mut() else {
                    continue;
                };
                for entry in &self.sender_directory_entries {
                    let item_id = LiveId::from_str(&entry.sender_profile_id);
                    let Some(item) = list.item(cx, item_id, id!(sender_entry)) else {
                        continue;
                    };
                    let mut item_scope = Scope::with_props(entry);
                    item.draw_all(cx, &mut item_scope);
                }
            }
        }
        DrawStep::done()
    }
}

impl BotfatherSettings {
    pub fn populate(&mut self, cx: &mut Cx, _selected_room_id: Option<String>) {
        let _ = botfather::ensure_loaded_for_current_user();
        let _ = botfather::refresh_inventory_from_rooms_list(cx);
        let defaults = botfather::default_config_form();

        self.view
            .text_input(ids!(crew_endpoint_input))
            .set_text(cx, &defaults.crew_endpoint);
        self.view
            .text_input(ids!(crew_auth_token_env_input))
            .set_text(cx, &defaults.crew_auth_token_env);
        self.view
            .text_input(ids!(openclaw_gateway_input))
            .set_text(cx, &defaults.openclaw_gateway_url);
        self.view
            .text_input(ids!(openclaw_auth_token_env_input))
            .set_text(cx, &defaults.openclaw_auth_token_env);
        self.view
            .text_input(ids!(workspace_root_input))
            .set_text(cx, &defaults.workspace_root);
        self.view
            .text_input(ids!(shared_sender_homeserver_input))
            .set_text(cx, &defaults.shared_sender_homeserver);
        self.view
            .text_input(ids!(shared_sender_user_id_input))
            .set_text(cx, &defaults.shared_sender_user_id);
        self.view
            .text_input(ids!(shared_sender_device_id_input))
            .set_text(cx, &defaults.shared_sender_device_id);
        self.view
            .text_input(ids!(shared_sender_access_token_env_input))
            .set_text(cx, &defaults.shared_sender_access_token_env);
        self.view
            .text_input(ids!(secure_sender_homeserver_input))
            .set_text(cx, &defaults.secure_sender_homeserver);
        self.view
            .text_input(ids!(secure_sender_user_id_input))
            .set_text(cx, &defaults.secure_sender_user_id);
        self.view
            .text_input(ids!(secure_sender_device_id_input))
            .set_text(cx, &defaults.secure_sender_device_id);
        self.view
            .text_input(ids!(secure_sender_access_token_env_input))
            .set_text(cx, &defaults.secure_sender_access_token_env);
        self.view
            .text_input(ids!(shared_sender_password_input))
            .set_text(cx, "");
        self.view
            .text_input(ids!(secure_sender_password_input))
            .set_text(cx, "");

        self.view
            .label(ids!(crew_runtime_summary_label))
            .set_text(cx, &botfather::runtime_summary(RuntimeKind::Crew));
        self.view
            .label(ids!(openclaw_runtime_summary_label))
            .set_text(cx, &botfather::runtime_summary(RuntimeKind::OpenClaw));
        self.view
            .label(ids!(bots_summary_label))
            .set_text(cx, &botfather::bots_overview());
        self.view
            .label(ids!(shared_sender_summary_label))
            .set_text(cx, &botfather::sender_profile_summary("shared-bot-sender"));
        self.view.label(ids!(secure_sender_summary_label)).set_text(
            cx,
            &botfather::sender_profile_summary("secure-room-bot-sender"),
        );
        self.sender_directory_entries = botfather::sender_directory_entries();
        self.binding_entries = botfather::explicit_room_binding_entries();
        self.view
            .label(ids!(bindings_stats_label))
            .set_text(cx, &botfather::bindings_stats_summary());
        let preview_enabled = botfather::room_stream_preview_enabled();
        self.view
            .label(ids!(room_stream_mode_summary_label))
            .set_text(
                cx,
                if preview_enabled {
                    "Preview mode is on. Bot output stays in the room panel until you click Post Preview."
                } else {
                    "Preview mode is off. Finished bot output is sent back to Matrix automatically."
                },
            );
        let command_help = format!(
            "{}\n\nexamples\n/bot create reviewer\n/bot bind reviewer\n/bot set-model reviewer deepseek-chat\n/bot set-prompt reviewer You are my reviewer\n/bot diagnostics",
            help_text()
        );
        self.view
            .label(ids!(command_hint_label))
            .set_text(cx, &command_help);
        self.view
            .label(ids!(workspace_summary_label))
            .set_text(cx, &botfather::workspace_overview());
        self.view
            .label(ids!(diagnostics_summary_label))
            .set_text(cx, &botfather::diagnostics_overview());
        apply_preview_toggle_style(
            &self.view.button(ids!(room_stream_mode_toggle)),
            cx,
            preview_enabled,
        );
        self.refresh_custom_sender_manager(cx);

        self.apply_section_state(cx);
        self.apply_bindings_filter_style(cx);
    }

    fn clear(&mut self, cx: &mut Cx) {
        self.binding_entries.clear();
        self.bindings_filter = BindingsFilter::All;
        self.custom_sender_choice_ids.clear();
        self.selected_custom_sender_profile_id = None;
        self.selected_custom_sender_security = SenderSecurityLevel::Standard;
        self.sender_directory_entries.clear();
        for input in [
            ids!(crew_endpoint_input),
            ids!(crew_auth_token_env_input),
            ids!(openclaw_gateway_input),
            ids!(openclaw_auth_token_env_input),
            ids!(workspace_root_input),
            ids!(shared_sender_homeserver_input),
            ids!(shared_sender_user_id_input),
            ids!(shared_sender_device_id_input),
            ids!(shared_sender_access_token_env_input),
            ids!(shared_sender_password_input),
            ids!(secure_sender_homeserver_input),
            ids!(secure_sender_user_id_input),
            ids!(secure_sender_device_id_input),
            ids!(secure_sender_access_token_env_input),
            ids!(secure_sender_password_input),
            ids!(custom_sender_profile_id_input),
            ids!(custom_sender_name_input),
            ids!(custom_sender_homeserver_input),
            ids!(custom_sender_user_id_input),
            ids!(custom_sender_device_id_input),
            ids!(custom_sender_access_token_env_input),
            ids!(custom_sender_password_input),
        ] {
            self.view.text_input(input).set_text(cx, "");
        }

        for label in [
            ids!(crew_runtime_summary_label),
            ids!(openclaw_runtime_summary_label),
            ids!(bots_summary_label),
            ids!(shared_sender_summary_label),
            ids!(secure_sender_summary_label),
            ids!(selected_custom_sender_summary_label),
            ids!(bindings_stats_label),
            ids!(room_stream_mode_summary_label),
            ids!(command_hint_label),
            ids!(workspace_summary_label),
            ids!(diagnostics_summary_label),
            ids!(status_label),
        ] {
            self.view.label(label).set_text(cx, "");
        }
    }

    fn set_status(&mut self, cx: &mut Cx, status: &str) {
        self.view.label(ids!(status_label)).set_text(cx, status);
    }

    fn set_section(&mut self, cx: &mut Cx, section: BotfatherSettingsSection) {
        self.selected_section = section;
        self.apply_section_state(cx);
    }

    fn apply_section_state(&mut self, cx: &mut Cx) {
        let runtimes_active = self.selected_section == BotfatherSettingsSection::Runtimes;
        let bots_active = self.selected_section == BotfatherSettingsSection::Bots;
        let senders_active = self.selected_section == BotfatherSettingsSection::Senders;
        let bindings_active = self.selected_section == BotfatherSettingsSection::Bindings;
        let workspace_active = self.selected_section == BotfatherSettingsSection::Workspace;
        let diagnostics_active = self.selected_section == BotfatherSettingsSection::Diagnostics;

        self.view
            .view(ids!(runtimes_section))
            .set_visible(cx, runtimes_active);
        self.view
            .view(ids!(bots_section))
            .set_visible(cx, bots_active);
        self.view
            .view(ids!(senders_section))
            .set_visible(cx, senders_active);
        self.view
            .view(ids!(bindings_section))
            .set_visible(cx, bindings_active);
        self.view
            .view(ids!(workspace_section))
            .set_visible(cx, workspace_active);
        self.view
            .view(ids!(diagnostics_section))
            .set_visible(cx, diagnostics_active);

        apply_section_button_style(
            &self.view.button(ids!(runtimes_section_button)),
            cx,
            runtimes_active,
        );
        apply_section_button_style(
            &self.view.button(ids!(bots_section_button)),
            cx,
            bots_active,
        );
        apply_section_button_style(
            &self.view.button(ids!(senders_section_button)),
            cx,
            senders_active,
        );
        apply_section_button_style(
            &self.view.button(ids!(bindings_section_button)),
            cx,
            bindings_active,
        );
        apply_section_button_style(
            &self.view.button(ids!(workspace_section_button)),
            cx,
            workspace_active,
        );
        apply_section_button_style(
            &self.view.button(ids!(diagnostics_section_button)),
            cx,
            diagnostics_active,
        );
        self.apply_runtime_cards_state(cx);
        self.apply_bindings_filter_style(cx);
    }

    fn apply_runtime_cards_state(&mut self, cx: &mut Cx) {
        self.view
            .view(ids!(crew_runtime_body))
            .set_visible(cx, self.crew_runtime_expanded);
        self.view
            .view(ids!(openclaw_runtime_body))
            .set_visible(cx, self.openclaw_runtime_expanded);
        self.view
            .view(ids!(crew_runtime_form))
            .set_visible(cx, self.crew_runtime_expanded);
        self.view
            .view(ids!(openclaw_runtime_form))
            .set_visible(cx, self.openclaw_runtime_expanded);
        self.view
            .view(ids!(shared_sender_body))
            .set_visible(cx, self.shared_sender_expanded);
        self.view
            .view(ids!(secure_sender_body))
            .set_visible(cx, self.secure_sender_expanded);
        self.view
            .view(ids!(shared_sender_form))
            .set_visible(cx, self.shared_sender_expanded);
        self.view
            .view(ids!(secure_sender_form))
            .set_visible(cx, self.secure_sender_expanded);

        apply_runtime_toggle_style(
            &self.view.button(ids!(crew_runtime_toggle)),
            cx,
            self.crew_runtime_expanded,
        );
        apply_runtime_toggle_style(
            &self.view.button(ids!(openclaw_runtime_toggle)),
            cx,
            self.openclaw_runtime_expanded,
        );
        apply_runtime_toggle_style(
            &self.view.button(ids!(shared_sender_toggle)),
            cx,
            self.shared_sender_expanded,
        );
        apply_runtime_toggle_style(
            &self.view.button(ids!(secure_sender_toggle)),
            cx,
            self.secure_sender_expanded,
        );
    }

    fn apply_bindings_filter_style(&mut self, cx: &mut Cx) {
        apply_section_button_style(
            &self.view.button(ids!(bindings_filter_all_button)),
            cx,
            self.bindings_filter == BindingsFilter::All,
        );
        apply_section_button_style(
            &self.view.button(ids!(bindings_filter_shared_button)),
            cx,
            self.bindings_filter == BindingsFilter::Shared,
        );
        apply_section_button_style(
            &self.view.button(ids!(bindings_filter_personal_button)),
            cx,
            self.bindings_filter == BindingsFilter::Personal,
        );
    }

    fn refresh_custom_sender_manager(&mut self, cx: &mut Cx) {
        let dropdown = self.view.drop_down(ids!(custom_sender_selector_dropdown));
        let mut choice_ids = vec![String::new()];
        let mut labels = vec!["Create New Sender".to_string()];

        for option in botfather::sender_profile_options().into_iter().filter(|option| {
            !matches!(
                option.sender_profile_id.as_str(),
                "current-user" | "shared-bot-sender" | "secure-room-bot-sender"
            )
        }) {
            choice_ids.push(option.sender_profile_id);
            labels.push(option.label);
        }

        self.custom_sender_choice_ids = choice_ids;
        dropdown.set_labels(cx, labels);

        let selected_index = self
            .selected_custom_sender_profile_id
            .as_ref()
            .and_then(|sender_profile_id| {
                self.custom_sender_choice_ids
                    .iter()
                    .position(|candidate| candidate == sender_profile_id)
            })
            .unwrap_or(0);
        dropdown.set_selected_item(cx, selected_index);

        let sender_profile_id = self
            .custom_sender_choice_ids
            .get(selected_index)
            .cloned()
            .unwrap_or_default();
        if sender_profile_id.is_empty() {
            self.load_custom_sender_form(cx, None);
        } else {
            self.load_custom_sender_form(cx, Some(&sender_profile_id));
        }
    }

    fn load_custom_sender_form(&mut self, cx: &mut Cx, sender_profile_id: Option<&str>) {
        let security_dropdown = self.view.drop_down(ids!(custom_sender_security_dropdown));
        security_dropdown.set_labels(cx, sender_security_level_labels());

        match sender_profile_id.and_then(botfather::sender_profile_form) {
            Some(form) => {
                self.selected_custom_sender_profile_id = Some(form.sender_profile_id.clone());
                self.selected_custom_sender_security = form.security;
                self.view
                    .text_input(ids!(custom_sender_profile_id_input))
                    .set_text(cx, &form.sender_profile_id);
                self.view
                    .text_input(ids!(custom_sender_name_input))
                    .set_text(cx, &form.name);
                self.view
                    .text_input(ids!(custom_sender_homeserver_input))
                    .set_text(cx, &form.homeserver_url);
                self.view
                    .text_input(ids!(custom_sender_user_id_input))
                    .set_text(cx, &form.matrix_user_id);
                self.view
                    .text_input(ids!(custom_sender_device_id_input))
                    .set_text(cx, &form.device_id);
                self.view
                    .text_input(ids!(custom_sender_access_token_env_input))
                    .set_text(cx, &form.access_token_env);
                self.view
                    .text_input(ids!(custom_sender_password_input))
                    .set_text(cx, "");
                self.view
                    .label(ids!(selected_custom_sender_summary_label))
                    .set_text(cx, &botfather::sender_profile_summary(&form.sender_profile_id));
                self.view
                    .button(ids!(delete_custom_sender_button))
                    .set_enabled(cx, form.can_delete);
                security_dropdown
                    .set_selected_item(cx, sender_security_level_index(form.security));
            }
            None => {
                self.selected_custom_sender_profile_id = None;
                self.selected_custom_sender_security = SenderSecurityLevel::Standard;
                self.view
                    .text_input(ids!(custom_sender_profile_id_input))
                    .set_text(cx, "");
                self.view
                    .text_input(ids!(custom_sender_name_input))
                    .set_text(cx, "");
                self.view
                    .text_input(ids!(custom_sender_homeserver_input))
                    .set_text(cx, "");
                self.view
                    .text_input(ids!(custom_sender_user_id_input))
                    .set_text(cx, "");
                self.view
                    .text_input(ids!(custom_sender_device_id_input))
                    .set_text(cx, "");
                self.view
                    .text_input(ids!(custom_sender_access_token_env_input))
                    .set_text(cx, "");
                self.view
                    .text_input(ids!(custom_sender_password_input))
                    .set_text(cx, "");
                self.view
                    .label(ids!(selected_custom_sender_summary_label))
                    .set_text(
                        cx,
                        "Create a sender profile, save it, then verify login to store a sender session.",
                    );
                self.view
                    .button(ids!(delete_custom_sender_button))
                    .set_enabled(cx, false);
                security_dropdown
                    .set_selected_item(cx, sender_security_level_index(SenderSecurityLevel::Standard));
            }
        }
    }

    fn filtered_binding_entries(&self) -> Vec<botfather::ExplicitRoomBindingEntry> {
        self.binding_entries
            .iter()
            .filter(|entry| match self.bindings_filter {
                BindingsFilter::All => true,
                BindingsFilter::Shared => entry.mode == "shared-room-bot",
                BindingsFilter::Personal => entry.mode == "personal-assist",
            })
            .cloned()
            .collect()
    }

    fn focus_sender_profile(&mut self, cx: &mut Cx, sender_profile_id: &str) {
        self.set_section(cx, BotfatherSettingsSection::Senders);
        match sender_profile_id {
            "shared-bot-sender" => {
                self.shared_sender_expanded = true;
                self.secure_sender_expanded = false;
            }
            "secure-room-bot-sender" => {
                self.shared_sender_expanded = false;
                self.secure_sender_expanded = true;
            }
            _ => {
                self.selected_custom_sender_profile_id = Some(sender_profile_id.to_string());
                self.refresh_custom_sender_manager(cx);
            }
        }
        self.apply_runtime_cards_state(cx);
        self.set_status(
            cx,
            &format!("Focused sender profile `{sender_profile_id}` in Senders."),
        );
    }
}

fn sender_security_level_labels() -> Vec<String> {
    vec![
        "Standard".to_string(),
        "Elevated".to_string(),
        "Isolated".to_string(),
    ]
}

fn sender_security_level_index(security: SenderSecurityLevel) -> usize {
    match security {
        SenderSecurityLevel::Standard => 0,
        SenderSecurityLevel::Elevated => 1,
        SenderSecurityLevel::Isolated => 2,
    }
}

fn sender_security_level_from_index(index: usize) -> SenderSecurityLevel {
    match index {
        1 => SenderSecurityLevel::Elevated,
        2 => SenderSecurityLevel::Isolated,
        _ => SenderSecurityLevel::Standard,
    }
}

fn normalize_sender_profile_id_input(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn apply_section_button_style(button: &ButtonRef, cx: &mut Cx, active: bool) {
    let (bg_color, fg_color) = if active {
        (
            crate::shared::styles::COLOR_ACTIVE_PRIMARY,
            crate::shared::styles::COLOR_PRIMARY,
        )
    } else {
        (
            crate::shared::styles::COLOR_BG_PREVIEW,
            vec4(0.109, 0.153, 0.298, 1.0),
        )
    };
    button.apply_over(
        cx,
        live! {
            draw_bg: {
                color: (bg_color)
            }
            draw_icon: {
                color: (fg_color)
            }
            draw_text: {
                color: (fg_color)
            }
        },
    );
}

fn apply_runtime_toggle_style(button: &ButtonRef, cx: &mut Cx, expanded: bool) {
    let (bg_color, fg_color, rotation_angle) = if expanded {
        (
            crate::shared::styles::COLOR_ACTIVE_PRIMARY,
            crate::shared::styles::COLOR_PRIMARY,
            180.0,
        )
    } else {
        (
            crate::shared::styles::COLOR_BG_PREVIEW,
            vec4(0.109, 0.153, 0.298, 1.0),
            90.0,
        )
    };
    button.apply_over(
        cx,
        live! {
            draw_bg: {
                color: (bg_color)
            }
            draw_icon: {
                color: (fg_color)
                rotation_angle: (rotation_angle)
            }
            draw_text: {
                color: (fg_color)
            }
        },
    );
}

fn apply_preview_toggle_style(button: &ButtonRef, cx: &mut Cx, enabled: bool) {
    let (bg_color, fg_color, text) = if enabled {
        (
            crate::shared::styles::COLOR_ACTIVE_PRIMARY,
            crate::shared::styles::COLOR_PRIMARY,
            "Disable Preview",
        )
    } else {
        (
            crate::shared::styles::COLOR_BG_PREVIEW,
            vec4(0.109, 0.153, 0.298, 1.0),
            "Enable Preview",
        )
    };
    button.set_text(cx, text);
    button.apply_over(
        cx,
        live! {
            draw_bg: {
                color: (bg_color)
            }
            draw_icon: {
                color: (fg_color)
            }
            draw_text: {
                color: (fg_color)
            }
        },
    );
}

impl BotfatherSettingsRef {
    pub fn populate(&self, cx: &mut Cx, selected_room_id: Option<String>) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.populate(cx, selected_room_id);
    }
}
