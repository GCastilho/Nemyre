#![allow(dead_code)] // TODO: Eventually this should be removed; I've added bc it was annoying

use rand::{random_iter, random_range};
use std::{
    collections::VecDeque,
    thread::sleep,
    time::{Duration, Instant},
};

const TICKS_PER_SECOND: u32 = 1000;

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

struct PreviousAction {
    with: usize,
    tick: u64,
}

struct Neuron {
    potential: f64,
    previous_actions: VecDeque<PreviousAction>,
    coordinates: Coordinates,
    axon: Vec<Synapse>,
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
    // TODO: Add easing function to change period soothly
    fn update(&mut self, tick: u64) -> bool {
        let period = match self.excitation {
            0 => return false, // Não dispara
            n => {
                let period = (u8::MAX as u32 * self.min_period) as f64 / n as f64;
                period.round() as u64
            }
        };
        if tick < self.last_tick_fired + period {
            return false; // Não dispara
        }

        // TODO: Implementar disparo. Da onde vem a força?
        /*
         * 2 Opções:
         *  - Chamar a fn q transmite a mensagem daqui
         *  - Retornar o valor da FORÇA da sinapse dessa fn e o caller transmitir a mensagem
         */
        println!("PLACEHOLDER PARA O DISPARO");
        self.last_tick_fired = tick;
        true
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

        // Seta a excitação dos receptores
        // Atualiza os receptores
        for (i, receptor) in receptors.iter_mut().enumerate() {
            receptor.excitation = random_range(118..138);
            if receptor.update(tick) {
                let connected_axons = &receptor.axon;
                // O valor do Potencial de Ação, multiplicado pelo weight, é enviado para o with
                println!(
                    "Receptor {i} mandando para {:?}",
                    connected_axons.iter().map(|v| v.with).collect::<Vec<_>>()
                );
            }
        }

        // Manda sinapses pros neurônios

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

        assert!(!receptor.update(1));
        assert!(receptor.update(50));
        assert_eq!(receptor.last_tick_fired, 50);
    }

    #[test]
    fn fire_half_excitation() {
        let max_frequency = 20;
        let mut receptor = SensoryReceptor::new(max_frequency, COORDINATES);
        receptor.excitation = 128;

        assert!(!receptor.update(50));
        assert!(receptor.update(100));
        assert_eq!(receptor.last_tick_fired, 100);
    }

    #[test]
    fn fire_74_excitation() {
        let max_frequency = 20;
        let mut receptor = SensoryReceptor::new(max_frequency, COORDINATES);
        receptor.excitation = 74;

        assert!(!receptor.update(171));
        assert!(receptor.update(172));
    }
}
