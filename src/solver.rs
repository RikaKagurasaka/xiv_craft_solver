use std::collections::VecDeque;
use std::ops::Not;
use crate::action::{Action, ACTIONS};
use crate::craft::Craft;
use crate::specs::Success;

macro_rules! action_vec {
    ($($tt:expr),*) => { vec![ $(Some(&$tt),)*]};
}
pub fn next_action_picker_1<'a>(craft: &Craft) -> Vec<Option<&'a Action>> {
    if craft.success != Success::Pending { return vec![None]; }
    let mut available_actions = Vec::<Option<&'a Action>>::new();
    let mut forbidden_actions = Vec::<Option<&'a Action>>::new();
    if craft.step_count == 0 { return action_vec![ACTIONS.muscle_memory]; }
    if craft.step_count == 1 { return action_vec![ACTIONS.manipulation]; }
    if craft.step_count == 2 { return action_vec![ACTIONS.veneration]; }
    if craft.step_count == 3 { available_actions.append(&mut action_vec![ACTIONS.waste_not_ii/*,ACTIONS.waste_not*/]) }
    if craft.buffs.waste_not > 0 || craft.buffs.muscle_memory > 0 { available_actions.append(&mut action_vec![ACTIONS.groundwork]) }
    if craft.buffs.muscle_memory > 0 { forbidden_actions.append(&mut action_vec![ACTIONS.basic_synthesis,ACTIONS.careful_synthesis,ACTIONS.prudent_synthesis,ACTIONS.delicate_synthesis]) }
    if craft.buffs.waste_not > 0 { forbidden_actions.append(&mut action_vec![ACTIONS.prudent_synthesis]) }
    available_actions.append(&mut action_vec![ACTIONS.basic_synthesis,ACTIONS.careful_synthesis,ACTIONS.prudent_synthesis,ACTIONS.delicate_synthesis]);
    let mut result_actions = Vec::<Option<&'a Action>>::new();
    for action in available_actions {
        if !forbidden_actions.contains(&action) && action.unwrap().can_use(craft) && result_actions.iter().any(|x| x.unwrap() == action.unwrap()).not() {
            result_actions.push(action);
        }
    }
    result_actions
}

pub fn generate_routes_phase1(craft: Craft) -> Vec<Craft> {
    let mut queue = Vec::new();
    queue.push(craft);
    let mut routes = Vec::new();
    while !queue.is_empty() {
        let craft = queue.pop().unwrap();
        for action in next_action_picker_1(&craft) {
            let mut craft = craft.clone();
            craft.run_action(action.unwrap());
            let remaining_prog = (craft.recipe.progress as f32 - craft.progression as f32) / craft.get_base_progression() as f32;
            if remaining_prog <= 2.0 {
                if remaining_prog <= 0.0 {
                    continue;
                } else if 0.0 < remaining_prog && remaining_prog <= 1.2 {
                    // pass
                } else if 1.2 < remaining_prog && remaining_prog <= 1.2 {
                    craft.cp -= 7;
                } else if 1.2 < remaining_prog && remaining_prog <= 2.0 {
                    craft.cp -= 12;
                }
                craft.durability -= 10;
                routes.push(craft);
                continue;
            }

            if craft.step_count < 8 { queue.push(craft); }
        }
    }
    routes
}


pub fn next_action_phase_2<'a>(craft: &Craft) -> Vec<Option<&'a Action>> {
    let mut available_actions = vec![&ACTIONS.basic_touch, &ACTIONS.prudent_touch, &ACTIONS.preparatory_touch];
    let mut forbidden_actions = Vec::new();
    if craft.success != Success::Pending { return vec![None]; }
    if craft.buffs.innovation > 0 {
        forbidden_actions.push(&ACTIONS.innovation);
    } else {
        if craft.buffs.inner_quiet >= 2 {
            forbidden_actions.append(&mut vec![&ACTIONS.basic_touch,
                                               &ACTIONS.standard_touch,
                                               &ACTIONS.advanced_touch,
                                               &ACTIONS.trained_finesse,
                                               &ACTIONS.prudent_touch,
                                               &ACTIONS.preparatory_touch,
                                               &ACTIONS.byregot_blessing]);
        }
        available_actions.push(&ACTIONS.innovation);
    }
    if craft.buffs.manipulation == 0 && craft.buffs.basic_touch == 0 && craft.buffs.standard_touch == 0 && craft.buffs.inner_quiet < 8 {
        available_actions.push(&ACTIONS.manipulation);
    }
    if craft.buffs.waste_not > 0 {
        available_actions.push(&ACTIONS.preparatory_touch);
        forbidden_actions.push(&ACTIONS.prudent_touch);
    } else {
        available_actions.push(&ACTIONS.prudent_touch);
        forbidden_actions.push(&ACTIONS.preparatory_touch);
    }
    if craft.buffs.basic_touch > 0 {
        available_actions.push(&ACTIONS.standard_touch);
        forbidden_actions.push(&ACTIONS.basic_touch);
    }
    if craft.buffs.standard_touch > 0 {
        available_actions.push(&ACTIONS.advanced_touch);
        forbidden_actions.push(&ACTIONS.basic_touch);
    }
    if craft.buffs.inner_quiet >= 10 {
        available_actions.push(&ACTIONS.trained_finesse);
        available_actions.push(&ACTIONS.great_strides);
    }
    if craft.buffs.great_strides > 0 {
        forbidden_actions.push(&ACTIONS.trained_finesse);
        forbidden_actions.push(&ACTIONS.great_strides);
        if craft.buffs.innovation > 0 {
            available_actions.push(&ACTIONS.byregot_blessing);
        }
    }
    let mut final_actions: Vec<Option<&Action>> = Vec::new();
    //drop duplicates

    for action in available_actions {
        if !forbidden_actions.contains(&action) && !final_actions.iter().any(|a| a.unwrap() == action) {
            final_actions.push(Some(action));
        }
    }
    final_actions
}

pub fn generate_routes_phase2(craft: Craft) -> Option<Craft> {
    let mut queue = VecDeque::new();
    queue.push_back(craft);
    let mut top_route: Option<Craft> = None;
    while !queue.is_empty() {
        let _craft = queue.pop_front().unwrap();
        for action in next_action_phase_2(&_craft) {
            if _craft.success != Success::Pending || action.is_none() || !action.unwrap().can_use(&_craft) {
                continue;
            }
            let mut craft = _craft.clone();
            craft.run_action(action.unwrap());
            if action.unwrap() == &ACTIONS.byregot_blessing {
                if let Some(top_route) = &mut top_route {
                    if top_route.quality < craft.quality {
                        *top_route = craft;
                    }
                } else {
                    top_route = Some(craft);
                }
            } else {
                queue.push_back(craft);
            }
        }
    }
    top_route
}