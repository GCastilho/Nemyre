#![allow(dead_code)] // TODO: Eventually this should be removed; I've added bc it was annoying

use rand::{random_iter, random_range};
use std::{
    collections::VecDeque,
    thread::sleep,
    time::{Duration, Instant},
};

const TICKS_PER_SECOND: u32 = 1000;
const MINIMUM_POTENTIAL: f64 = 100.0;

struct Coordinates {
    x: f64,
    y: f64,
    z: f64,
}

/// Conexão entre neurônios (e de neurônios <-> nervos)
#[derive(Debug, Clone)]
struct Synapse {
    with: usize,
    weight: f64, // TODO: Max value is 1
}

/// Signal received on last tick
#[derive(Debug)]
struct Action {
    with: usize,
    value: f64,
}

/// Action processed/received in a previous tick, kept for Synapse adjustment upon new Action
struct PreviousAction {
    with: usize,
    tick: u64,
}

struct Neuron {
    potential: f64,
    actions: Vec<Action>,
    previous_actions: VecDeque<PreviousAction>,
    coordinates: Coordinates,
    axon: Vec<Synapse>,
}

impl Neuron {
    // TODO: A força da conexão é controlada pelo recebedor, não por quem envia. Quem envia controla
    // a força do disparo
    pub fn update(&mut self, tick: u64) {
        let potential_received = self.actions.iter().map(|a| a.value).sum::<f64>();
        self.potential += potential_received;

        if self.potential < MINIMUM_POTENTIAL {
            return;
        }

        self.actions
            .iter()
            .map(|a| PreviousAction { with: a.with, tick })
            .for_each(|pa| self.previous_actions.push_back(pa));
    }
}

/// Input receptor from the system
///
/// Converts excitation value into synapses to connected neurons. For now that is linear
/// TODO: Será necessário poder setar a frequência ou os pulsos diretamente (ex: pulsos randômicos)
struct SensoryReceptor {
    excitation: u8, // TODO: Esse é o melhor nome?
    coordinates: Coordinates,
    axon: Vec<Synapse>,
    last_tick_fired: u64,
    min_period: u32,
}

impl SensoryReceptor {
    // TODO: max_frequency customizável por neurônio assim como a forma de onda
    pub fn new(max_frequency: u32, coordinates: Coordinates) -> Self {
        Self {
            excitation: 0,
            min_period: TICKS_PER_SECOND / max_frequency, // TODO: Talvez arredondar, não truncar
            coordinates,
            axon: Vec::new(),
            last_tick_fired: 0,
        }
    }

    // TODO: O ideal é que o tick seja acessível globalmente de alguma forma
    // TODO: N sei se update é o melhor nome
    // TODO: Add easing function to change period smoothly
    /// Process receptor excitation into an action strength
    ///
    /// Returns [`None`] if receptor shouldn't fire
    /// Returns [`Some(t)`] with the strength the receptor fired the Action Potential
    fn update(&mut self, tick: u64) -> Option<f64> {
        match self.excitation {
            0 => None,
            n => {
                let period = (u8::MAX as u32 * self.min_period) as f64 / n as f64;
                Some(period.round() as u64)
            }
        }
        .filter(|period| self.last_tick_fired + period <= tick)
        .map(|_| {
            self.last_tick_fired = tick;
            // TODO: Implementar disparo. Da onde vem a força?
            1.0
        })
    }
}

/// Output neurons from the system
///
/// Receive synapses and convert that into an excitation value
struct MotorNerve {
    excitation: u8, // TODO: Esse é o melhor nome?
    coordinates: Coordinates,
}

fn main() {
    let mut receptors = Vec::new();
    for _ in 0..110 {
        let coordinates = Coordinates {
            x: random_range(0.0..10.0),
            y: random_range(0.0..10.0),
            z: random_range(0.0..10.0),
        };
        let mut axon = random_iter()
            .filter(|v| *v < 1000)
            .take(11)
            .collect::<Vec<u16>>();
        axon.dedup();
        let axon = axon
            .into_iter()
            .map(|with| Synapse {
                with: with as usize,
                weight: random_range(0.0..1.0),
            })
            .collect();
        let mut receptor = SensoryReceptor::new(20, coordinates);
        receptor.axon = axon;
        receptors.push(receptor);
    }

    let mut neurons = Vec::new();
    for _ in 0..1000 {
        let coordinates = Coordinates {
            x: random_range(0.0..10.0),
            y: random_range(0.0..10.0),
            z: random_range(0.0..10.0),
        };
        let mut axon = random_iter()
            .filter(|v| *v < 1000)
            .take(110)
            .collect::<Vec<u16>>();
        axon.dedup();
        let axon = axon
            .into_iter()
            .map(|with| Synapse {
                with: with as usize,
                weight: random_range(0.0..1.0),
            })
            .collect();

        let neuron = Neuron {
            potential: 0.0,
            actions: Vec::new(),
            previous_actions: VecDeque::new(),
            coordinates,
            axon,
        };
        neurons.push(neuron);
    }

    let mut tick: u64 = 0;
    let tick_min_duration = Duration::from_millis(TICKS_PER_SECOND as u64 / 1000);
    loop {
        tick += 1;
        let now = Instant::now();

        // TODO: Array de actions deve ser limpo qdo o neurônio processá-las
        let mut fired_actions = Vec::new();

        // Seta a excitação dos receptores
        // Atualiza os receptores
        for receptor in &mut receptors {
            receptor.excitation = random_range(118..138);
            if let Some(strength) = receptor.update(tick) {
                // O valor do Potencial de Ação, multiplicado pelo weight, é enviado para o with
                let actions = receptor.axon.iter().map(|a| Action {
                    with: a.with,
                    value: a.weight * strength,
                });
                fired_actions.extend(actions);
            }
        }

        // Processa os neurônios
        for neuron in &mut neurons {
            neuron.update(tick);
        }

        // Manda sinapses pros neurônios
        for action in fired_actions {
            let neuron = neurons
                .get_mut(action.with)
                .unwrap_or_else(|| panic!("Fail to find neuron for action {:?}", action));
            neuron.actions.push(action);
        }

        let tick_duration_left = tick_min_duration
            .checked_sub(now.elapsed())
            .unwrap_or_default();
        sleep(tick_duration_left);
    }
}

#[cfg(test)]
mod sensory_tests {
    use crate::{Coordinates, SensoryReceptor};

    const COORDINATES: Coordinates = Coordinates {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    #[test]
    fn fire_on_max_excitation() {
        let max_frequency = 20;
        let mut receptor = SensoryReceptor::new(max_frequency, COORDINATES);
        receptor.excitation = 255;

        assert!(receptor.update(1).is_none());
        assert!(receptor.update(50).is_some());
        assert_eq!(receptor.last_tick_fired, 50);
    }

    #[test]
    fn fire_half_excitation() {
        let max_frequency = 20;
        let mut receptor = SensoryReceptor::new(max_frequency, COORDINATES);
        receptor.excitation = 128;

        assert!(receptor.update(50).is_none());
        assert!(receptor.update(100).is_some());
        assert_eq!(receptor.last_tick_fired, 100);
    }

    #[test]
    fn fire_74_excitation() {
        let max_frequency = 20;
        let mut receptor = SensoryReceptor::new(max_frequency, COORDINATES);
        receptor.excitation = 74;

        assert!(receptor.update(171).is_none());
        assert!(receptor.update(172).is_some());
    }
}
