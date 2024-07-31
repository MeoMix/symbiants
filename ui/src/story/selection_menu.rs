use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use rendering::common::selection::SelectedEntity;

use simulation::{
    common::{
        ant::{hunger::Hunger, AntInventory, AntName, AntRole, Dead},
        element::Element,
        pheromone::{Pheromone, PheromoneStrength},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
    nest_simulation::{
        ant::{birthing::Birthing, sleep::Asleep},
        nest::AtNest,
    },
};

#[derive(Component, Default, PartialEq, Copy, Clone, Debug)]
pub struct Selected;

pub fn update_selection_menu(
    mut contexts: EguiContexts,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    selected_ant_query: Query<(
        &Hunger,
        &AntName,
        &AntRole,
        &AntInventory,
        Option<&Birthing>,
        Option<&Dead>,
        Option<&Asleep>,
    )>,
    selected_element_query: Query<(&Element, &Position, Option<&AtNest>, Option<&AtCrater>)>,
    pheromone_query: Query<(
        &Position,
        &Pheromone,
        &PheromoneStrength,
        Option<&AtNest>,
        Option<&AtCrater>,
    )>,
    elements_query: Query<&Element>,
    selected_entity: Res<SelectedEntity>,
) {
    let window = primary_window_query.single();
    let ctx = contexts.ctx_mut();

    let selected_entity = match selected_entity.0 {
        Some(entity) => entity,
        None => return,
    };

    let selected_element = selected_element_query.get(selected_entity);
    let selected_ant = selected_ant_query.get(selected_entity);

    if selected_element.is_err() && selected_ant.is_err() {
        return;
    }

    egui::Window::new("Selection")
        .default_pos(egui::Pos2::new(0.0, window.height()))
        .resizable(false)
        .show(ctx, |ui| {
            if let Ok((element, element_position, element_at_nest, element_at_crater)) =
                selected_element
            {
                ui.label("Element");
                ui.label(&format!("Type: {:?}", element));

                // TODO: This is weird because really the "Pheromone" is selected not necessarily the Element?
                // TODO: This shows pheromones for unrelated inactive zone - want to match on zone
                for (
                    pheromone_position,
                    pheromone,
                    pheromone_strength,
                    pheromone_at_nest,
                    pheromone_at_crater,
                ) in pheromone_query.iter()
                {
                    // TODO: This is such a dumb way to confirm that pheromone data is being shown for the relevant UI
                    // Could subscribe selection_menu twice and use generics / no-op for irrelevant one as an alternative
                    // but ideally would just work off of selection itself
                    if pheromone_position == element_position
                        && ((element_at_nest.is_some() && pheromone_at_nest.is_some())
                            || (element_at_crater.is_some() && pheromone_at_crater.is_some()))
                    {
                        ui.label(&format!("Pheromone Type: {:?}", pheromone));
                        ui.label(&format!(
                            "Pheromone Strength: {:.0}",
                            pheromone_strength.value()
                        ));
                    }
                }
            } else if let Ok((hunger, name, ant_role, inventory, birthing, dead, asleep)) =
                selected_ant
            {
                ui.label("Ant");
                ui.label(&format!("Name: {}", name.0));
                ui.label(&format!("Role: {:?}", ant_role));
                ui.label(&format!("Hunger: {:.0}%", hunger.value()));

                if let Some(element_entity) = inventory.0 {
                    let element = elements_query.get(element_entity).unwrap();
                    ui.label(&format!("Carrying: {:?}", element));
                }

                if let Some(birthing) = birthing {
                    ui.label(&format!("Birthing: {:.0}%", birthing.value()));
                }

                if let Some(_) = asleep {
                    ui.label(&format!("Sleeping"));
                }

                if let Some(_) = dead {
                    // TODO: Maybe have it say "Died at XXX"
                    ui.label("Dead");
                }
            }
        });
}
