use anyhow::{Context, Result};
use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};
use rusqlite::Connection;

use crate::app;
use crate::gui::{ActionRow, Message, ToolsGui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DamageTypeChoice {
    Arcane,
    Physical,
}

impl DamageTypeChoice {
    fn label(self) -> &'static str {
        match self {
            DamageTypeChoice::Arcane => "Arcane",
            DamageTypeChoice::Physical => "Physical",
        }
    }

    fn all() -> [DamageTypeChoice; 2] {
        [DamageTypeChoice::Arcane, DamageTypeChoice::Physical]
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "Arcane" => Some(DamageTypeChoice::Arcane),
            "Physical" => Some(DamageTypeChoice::Physical),
            _ => None,
        }
    }
}

impl std::fmt::Display for DamageTypeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SkillChoice {
    Strength,
    Focus,
    Intelligence,
    Knowledge,
    Agility,
}

impl SkillChoice {
    fn label(self) -> &'static str {
        match self {
            SkillChoice::Strength => "Strength",
            SkillChoice::Focus => "Focus",
            SkillChoice::Intelligence => "Intelligence",
            SkillChoice::Knowledge => "Knowledge",
            SkillChoice::Agility => "Agility",
        }
    }

    fn all() -> [SkillChoice; 5] {
        [
            SkillChoice::Strength,
            SkillChoice::Focus,
            SkillChoice::Intelligence,
            SkillChoice::Knowledge,
            SkillChoice::Agility,
        ]
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "Strength" => Some(SkillChoice::Strength),
            "Focus" => Some(SkillChoice::Focus),
            "Intelligence" => Some(SkillChoice::Intelligence),
            "Knowledge" => Some(SkillChoice::Knowledge),
            "Agility" => Some(SkillChoice::Agility),
            _ => None,
        }
    }
}

impl std::fmt::Display for SkillChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActionTypeChoice {
    Interaction,
    Attack,
}

impl ActionTypeChoice {
    fn label(self) -> &'static str {
        match self {
            ActionTypeChoice::Interaction => "Interaction",
            ActionTypeChoice::Attack => "Attack",
        }
    }

    fn all() -> [ActionTypeChoice; 2] {
        [ActionTypeChoice::Interaction, ActionTypeChoice::Attack]
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "Interaction" => Some(ActionTypeChoice::Interaction),
            "Attack" => Some(ActionTypeChoice::Attack),
            _ => None,
        }
    }
}

impl std::fmt::Display for ActionTypeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

pub fn list_actions(conn: &Connection) -> Result<Vec<ActionRow>> {
    let actions = app::cards::action::list_cards(conn).context("list actions")?;
    Ok(actions
        .into_iter()
        .map(|a| ActionRow {
            name: a.name,
            action_point_cost: a.action_point_cost,
            action_type: match a.action_type {
                app::cards::action::ActionType::Interaction => "Interaction".to_string(),
                app::cards::action::ActionType::Attack => "Attack".to_string(),
            },
            text: a.text,
        })
        .collect())
}

pub fn view(app_state: &ToolsGui) -> Element<'_, Message> {
    let create_form = create_form_view(app_state);

    let list = actions_list_view(app_state);

    container(column![create_form, iced::widget::horizontal_rule(1), list].spacing(12))
        .width(Length::Fill)
        .into()
}

fn create_form_view(app_state: &ToolsGui) -> Element<'_, Message> {
    let action_type_choice = ActionTypeChoice::from_str(&app_state.action_type);

    let attack_damage_type_choice = DamageTypeChoice::from_str(&app_state.attack_damage_type);
    let attack_skill_choice = SkillChoice::from_str(&app_state.attack_skill);

    let interaction_skill_choice = SkillChoice::from_str(&app_state.interaction_skill);

    let attack_section = container(
        column![
            text("Attack").size(18),
            row![
                text("Damage").width(Length::Fixed(140.0)),
                text_input("e.g. 3", &app_state.attack_damage)
                    .on_input(Message::AttackDamageChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Damage Type").width(Length::Fixed(140.0)),
                pick_list(
                    DamageTypeChoice::all(),
                    attack_damage_type_choice,
                    |choice| Message::AttackDamageTypeChanged(choice.label().to_string())
                )
                .placeholder("Select...")
                .width(Length::Fixed(200.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Skill").width(Length::Fixed(140.0)),
                pick_list(SkillChoice::all(), attack_skill_choice, |choice| {
                    Message::AttackSkillChanged(choice.label().to_string())
                })
                .placeholder("Select...")
                .width(Length::Fixed(200.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Target (1-14)").width(Length::Fixed(140.0)),
                text_input("e.g. 10", &app_state.attack_target)
                    .on_input(Message::AttackTargetChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Range").width(Length::Fixed(140.0)),
                text_input("e.g. 1", &app_state.attack_range)
                    .on_input(Message::AttackRangeChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                button("Save Attack").on_press(Message::SaveAttackEdits),
                button("Delete Attack").on_press(Message::DeleteAttack),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(8),
        ]
        .spacing(10),
    )
    .padding(10)
    .width(Length::Fill);

    let interaction_section = container(
        column![
            text("Interaction").size(18),
            row![
                text("Range").width(Length::Fixed(140.0)),
                text_input("e.g. 2", &app_state.interaction_range)
                    .on_input(Message::InteractionRangeChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Skill").width(Length::Fixed(140.0)),
                pick_list(SkillChoice::all(), interaction_skill_choice, |choice| {
                    Message::InteractionSkillChanged(choice.label().to_string())
                })
                .placeholder("Select...")
                .width(Length::Fixed(200.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Target (1-14, optional)").width(Length::Fixed(140.0)),
                text_input("blank = NULL", &app_state.interaction_target)
                    .on_input(Message::InteractionTargetChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                button("Save Interaction").on_press(Message::SaveInteractionEdits),
                button("Delete Interaction").on_press(Message::DeleteInteraction),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(8),
        ]
        .spacing(10),
    )
    .padding(10)
    .width(Length::Fill);

    let subtype_section = match action_type_choice {
        Some(ActionTypeChoice::Attack) => Some(attack_section),
        Some(ActionTypeChoice::Interaction) => Some(interaction_section),
        None => None,
    };

    let form = column![
        text("Create Action").size(20),
        row![
            text("Name").width(Length::Fixed(140.0)),
            text_input("e.g. Strike", &app_state.action_name)
                .on_input(Message::ActionNameChanged)
                .width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
        row![
            text("Action Point Cost").width(Length::Fixed(140.0)),
            text_input("e.g. 2", &app_state.action_point_cost)
                .on_input(Message::ActionPointCostChanged)
                .width(Length::Fixed(120.0)),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
        row![
            text("Action Type").width(Length::Fixed(140.0)),
            pick_list(ActionTypeChoice::all(), action_type_choice, |choice| {
                Message::ActionTypeChanged(choice.label().to_string())
            })
            .placeholder("Select...")
            .width(Length::Fixed(200.0)),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
        row![
            text("Text").width(Length::Fixed(140.0)),
            text_input("Rules text / description", &app_state.action_text)
                .on_input(Message::ActionTextChanged)
                .width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
        row![
            button("Create").on_press(Message::CreateAction),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12),
    ]
    .spacing(10);

    let mut out = column![container(form).width(Length::Fill)].spacing(10);

    if let Some(sub) = subtype_section {
        out = out
            .push(iced::widget::horizontal_rule(1))
            .push(text("Subtype (create after saving the Action)").size(16))
            .push(sub);
    }

    container(out).width(Length::Fill).into()
}

fn actions_list_view(app_state: &ToolsGui) -> Element<'_, Message> {
    let header = row![
        text("Actions").size(20),
        iced::widget::Space::with_width(Length::Fill),
        text(format!("{} total", app_state.actions.len())).size(14),
    ]
    .align_items(Alignment::Center);

    let mut rows = column![].spacing(8);

    for a in &app_state.actions {
        let line1 = row![
            text(&a.name).size(16),
            iced::widget::Space::with_width(Length::Fill),
            text(format!("AP {}", a.action_point_cost)).size(14),
            text(&a.action_type).size(14),
        ]
        .spacing(12)
        .align_items(Alignment::Center);

        let line2 = row![text(&a.text).size(14)]
            .spacing(12)
            .align_items(Alignment::Center);

        let controls = row![
            button("Edit").on_press(Message::EditAction(a.name.clone())),
            button("Delete").on_press(Message::DeleteAction(a.name.clone())),
        ]
        .spacing(8)
        .align_items(Alignment::Center);

        rows = rows.push(
            container(column![line1, line2, controls].spacing(6))
                .padding(10)
                .width(Length::Fill),
        );
    }

    let content = column![header, rows].spacing(12);

    container(content).width(Length::Fill).into()
}

pub fn edit_view<'a>(app_state: &'a ToolsGui, original_name: &'a str) -> Element<'a, Message> {
    let action_type_choice = ActionTypeChoice::from_str(&app_state.action_type);

    let attack_damage_type_choice = DamageTypeChoice::from_str(&app_state.attack_damage_type);
    let attack_skill_choice = SkillChoice::from_str(&app_state.attack_skill);

    let interaction_skill_choice = SkillChoice::from_str(&app_state.interaction_skill);

    let attack_section = container(
        column![
            text("Attack").size(18),
            row![
                text("Damage").width(Length::Fixed(140.0)),
                text_input("e.g. 3", &app_state.attack_damage)
                    .on_input(Message::AttackDamageChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Damage Type").width(Length::Fixed(140.0)),
                pick_list(
                    DamageTypeChoice::all(),
                    attack_damage_type_choice,
                    |choice| Message::AttackDamageTypeChanged(choice.label().to_string())
                )
                .placeholder("Select...")
                .width(Length::Fixed(200.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Skill").width(Length::Fixed(140.0)),
                pick_list(SkillChoice::all(), attack_skill_choice, |choice| {
                    Message::AttackSkillChanged(choice.label().to_string())
                })
                .placeholder("Select...")
                .width(Length::Fixed(200.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Target (1-14)").width(Length::Fixed(140.0)),
                text_input("e.g. 10", &app_state.attack_target)
                    .on_input(Message::AttackTargetChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Range").width(Length::Fixed(140.0)),
                text_input("e.g. 1", &app_state.attack_range)
                    .on_input(Message::AttackRangeChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                button("Save Attack").on_press(Message::SaveAttackEdits),
                button("Delete Attack").on_press(Message::DeleteAttack),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(8),
        ]
        .spacing(10),
    )
    .padding(10)
    .width(Length::Fill);

    let interaction_section = container(
        column![
            text("Interaction").size(18),
            row![
                text("Range").width(Length::Fixed(140.0)),
                text_input("e.g. 2", &app_state.interaction_range)
                    .on_input(Message::InteractionRangeChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Skill").width(Length::Fixed(140.0)),
                pick_list(SkillChoice::all(), interaction_skill_choice, |choice| {
                    Message::InteractionSkillChanged(choice.label().to_string())
                })
                .placeholder("Select...")
                .width(Length::Fixed(200.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                text("Target (1-14, optional)").width(Length::Fixed(140.0)),
                text_input("blank = NULL", &app_state.interaction_target)
                    .on_input(Message::InteractionTargetChanged)
                    .width(Length::Fixed(120.0)),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(12)
            .align_items(Alignment::Center),
            row![
                button("Save Interaction").on_press(Message::SaveInteractionEdits),
                button("Delete Interaction").on_press(Message::DeleteInteraction),
                iced::widget::Space::with_width(Length::Fill),
            ]
            .spacing(8),
        ]
        .spacing(10),
    )
    .padding(10)
    .width(Length::Fill);

    let subtype_section = match action_type_choice {
        Some(ActionTypeChoice::Attack) => Some(attack_section),
        Some(ActionTypeChoice::Interaction) => Some(interaction_section),
        None => None,
    };

    let action_type_row = match action_type_choice {
        Some(choice) => row![
            text("Action Type").width(Length::Fixed(140.0)),
            text(choice.label()).size(14),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
        None => row![
            text("Action Type").width(Length::Fixed(140.0)),
            text("(invalid)").size(14),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
    };

    let mut form = column![
        text(format!("Edit Action: {}", original_name)).size(20),
        row![
            text("Name").width(Length::Fixed(140.0)),
            text_input("Action name", &app_state.action_name)
                .on_input(Message::ActionNameChanged)
                .width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
        row![
            text("Action Point Cost").width(Length::Fixed(140.0)),
            text_input("e.g. 2", &app_state.action_point_cost)
                .on_input(Message::ActionPointCostChanged)
                .width(Length::Fixed(120.0)),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
        action_type_row,
        row![
            text("Text").width(Length::Fixed(140.0)),
            text_input("Rules text / description", &app_state.action_text)
                .on_input(Message::ActionTextChanged)
                .width(Length::Fill),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
        row![
            button("Save Action").on_press(Message::SaveActionEdits),
            button("Cancel").on_press(Message::CancelEdit),
            iced::widget::Space::with_width(Length::Fill),
        ]
        .spacing(8),
    ]
    .spacing(10);

    if let Some(sub) = subtype_section {
        form = form.push(iced::widget::horizontal_rule(1)).push(sub);
    }

    container(form).width(Length::Fill).into()
}
